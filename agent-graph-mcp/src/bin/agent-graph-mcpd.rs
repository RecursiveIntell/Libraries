//! Long-lived owner daemon for durable agent-graph MCP operation.
//!
//! The daemon owns the data-directory lock, SQLite lifecycle, private Unix
//! listener, and one MCP service per accepted local proxy connection. Proxies
//! exchange bounded length-prefixed frames; rmcp receives the same payloads as
//! newline-delimited JSON-RPC over an in-process bridge.

use std::path::Path;

use agent_graph_mcp::{
    cli::{self, CliConfig},
    daemon, fs_security,
    proxy::{self, ProxyError},
    AgentGraphServer,
};
use rmcp::ServiceExt;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

fn main() {
    if let Err(error) = run() {
        eprintln!("agent-graph-mcpd: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let config = match cli::parse_args(&args) {
        Ok(config) => config,
        Err(error) => {
            if !error.message.is_empty() {
                eprintln!("agent-graph-mcpd: {}", error.message);
            }
            std::process::exit(error.exit_code);
        }
    };

    if config.ephemeral || config.data_dir.is_none() {
        return Err(
            "MODE_REQUIRED: agent-graph-mcpd requires --data-dir and cannot run in ephemeral mode"
                .into(),
        );
    }

    let runtime = tokio::runtime::Runtime::new().map_err(|error| error.to_string())?;
    runtime.block_on(run_async(config))
}

async fn run_async(config: CliConfig) -> Result<(), String> {
    let mut config = config;
    let data_dir = config
        .data_dir
        .clone()
        .ok_or_else(|| "MODE_REQUIRED: daemon data directory is required".to_string())?;
    let runtime_dir = cli::resolve_runtime_dir(&config).map_err(|error| error.to_string())?;
    let integrity_key_path = cli::resolve_integrity_key_path(&config);
    validate_integrity_key(&config, &data_dir, integrity_key_path.as_deref())?;
    config.integrity_key_path = integrity_key_path;

    let binary_digest = daemon::executable_digest();
    let (lock, connection) = daemon::open_owned(&data_dir, &binary_digest)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    let instance_id = format!("{}-{}", config.instance, std::process::id());
    daemon::record_instance_start(&connection, &instance_id, &binary_digest)
        .map_err(|error| format!("failed to record daemon start: {error}"))?;

    let socket_path = match daemon::prepare_socket_path(&runtime_dir, &config.instance)
        .and_then(|path| daemon::bind_private_socket(&path).map(|listener| (path, listener)))
    {
        Ok((path, listener)) => (path, listener),
        Err(error) => {
            let _ = daemon::record_instance_stop(&connection, &instance_id);
            return Err(format!("failed to prepare daemon socket: {error}"));
        }
    };
    let (socket_path, listener) = socket_path;

    let loop_result = accept_connections(&listener, &config).await;
    drop(listener);
    let socket_cleanup = remove_socket(&socket_path);
    let stop_result = daemon::record_instance_stop(&connection, &instance_id)
        .map_err(|error| format!("failed to record daemon stop: {error}"));
    drop(connection);
    drop(lock);

    loop_result?;
    socket_cleanup?;
    stop_result
}

fn validate_integrity_key(
    config: &CliConfig,
    data_dir: &Path,
    integrity_key_path: Option<&Path>,
) -> Result<(), String> {
    if !config.require_integrity_key {
        return Ok(());
    }

    let key_path = integrity_key_path.ok_or_else(|| {
        "INTEGRITY_KEY_REQUIRED: --require-integrity-key needs --integrity-key or AGENT_GRAPH_INTEGRITY_KEY_PATH".to_string()
    })?;
    fs_security::check_private_file(&key_path, true)
        .map_err(|error| format!("integrity key is not private/readable: {error}"))?;
    let length = std::fs::metadata(&key_path)
        .map_err(|error| format!("integrity key metadata unavailable: {error}"))?
        .len();
    if length < 32 {
        return Err("INTEGRITY_KEY_REQUIRED: integrity key must contain at least 32 bytes".into());
    }
    fs_security::validate_data_store(data_dir, Some(&key_path))
        .map_err(|error| format!("data store security check failed: {error}"))
}

async fn wait_for_shutdown_signal() -> Result<(), String> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm = signal(SignalKind::terminate())
            .map_err(|error| format!("failed to install SIGTERM handler: {error}"))?;
        let mut sigint = signal(SignalKind::interrupt())
            .map_err(|error| format!("failed to install SIGINT handler: {error}"))?;
        tokio::select! {
            _ = sigterm.recv() => Ok(()),
            _ = sigint.recv() => Ok(()),
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .map_err(|error| format!("daemon shutdown signal failed: {error}"))
    }
}

async fn accept_connections(listener: &UnixListener, config: &CliConfig) -> Result<(), String> {
    let mut shutdown = Box::pin(wait_for_shutdown_signal());
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, _) = accepted.map_err(|error| format!("daemon accept failed: {error}"))?;
                match fs_security::peer_uid_matches_current(&stream) {
                    Ok(true) => {
                        let connection_config = config.clone();
                        tokio::spawn(async move {
                            if let Err(error) = serve_connection(stream, connection_config).await {
                                tracing::debug!(%error, "daemon client connection closed");
                            }
                        });
                    }
                    Ok(false) => {
                        tracing::warn!("rejecting daemon client with a foreign peer UID");
                    }
                    Err(error) => {
                        tracing::warn!(%error, "rejecting daemon client whose UID could not be verified");
                    }
                }
            }
            signal = &mut shutdown => {
                signal.map_err(|error| format!("daemon shutdown signal failed: {error}"))?;
                break;
            }
        }
    }
    Ok(())
}

async fn serve_connection(stream: UnixStream, config: CliConfig) -> Result<(), String> {
    let (socket_reader, socket_writer) = stream.into_split();
    let (bridge, service_io) = tokio::io::duplex(proxy::MAX_FRAME.saturating_mul(2));
    let (bridge_reader, bridge_writer) = tokio::io::split(bridge);
    let (service_reader, service_writer) = tokio::io::split(service_io);

    let mut ingress = tokio::spawn(forward_frames_to_mcp(socket_reader, bridge_writer));
    let mut egress = tokio::spawn(forward_mcp_to_frames(bridge_reader, socket_writer));

    let server = AgentGraphServer::new_with_checkpoint_db(
        config.base_url,
        config.default_model,
        config.api_key,
        config.data_dir,
        config.integrity_key_path,
        config.checkpoint_db_path,
    )
    .map_err(|error| format!("failed to initialize MCP service: {error}"));

    let server = match server {
        Ok(server) => server,
        Err(error) => {
            ingress.abort();
            egress.abort();
            return Err(error);
        }
    };

    let running = server
        .serve((service_reader, service_writer))
        .await
        .map_err(|error| format!("MCP service initialization failed: {error}"));
    let running = match running {
        Ok(running) => running,
        Err(error) => {
            ingress.abort();
            egress.abort();
            return Err(error);
        }
    };

    tokio::select! {
        result = running.waiting() => {
            result.map_err(|error| format!("MCP service task failed: {error}"))?;
        }
        result = &mut ingress => {
            let result = result.map_err(|error| format!("proxy ingress task failed: {error}"))?;
            result.map_err(|error| format!("proxy ingress failed: {error}"))?;
        }
        result = &mut egress => {
            let result = result.map_err(|error| format!("proxy egress task failed: {error}"))?;
            result.map_err(|error| format!("proxy egress failed: {error}"))?;
        }
    }

    ingress.abort();
    egress.abort();
    Ok(())
}

async fn forward_frames_to_mcp<R, W>(mut reader: R, mut writer: W) -> Result<(), ProxyError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    loop {
        let payload = proxy::read_frame_async(&mut reader).await?;
        writer.write_all(&payload).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }
}

async fn forward_mcp_to_frames<R, W>(reader: R, mut writer: W) -> Result<(), ProxyError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut reader = BufReader::new(reader);
    while let Some(line) = proxy::read_bounded_line(&mut reader).await? {
        if line.is_empty() {
            continue;
        }
        proxy::write_frame_async(&mut writer, &line).await?;
    }
    Ok(())
}

fn remove_socket(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("failed to remove daemon socket: {error}")),
    }
}
