use agent_graph_mcp::{daemon, transport, AgentGraphServer};
use rmcp::ServiceExt;
use std::{
    os::unix::fs::PermissionsExt,
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut data = PathBuf::from("/tmp/agent-graph");
    let mut socket = PathBuf::from("/tmp/agent-graph/mcp.sock");
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--data-dir" => data = PathBuf::from(args.next().ok_or("missing data dir")?),
            "--socket" => socket = PathBuf::from(args.next().ok_or("missing socket")?),
            "--help" => {
                println!("agent-graph-mcpd --data-dir PATH --socket PATH");
                return Ok(());
            }
            "--version" => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            _ => return Err("unknown daemon argument".into()),
        }
    }
    std::fs::create_dir_all(&data)?;
    let (_lock, conn) = daemon::open_owned(&data, "daemon")?;
    let id = daemon::identity(&conn)?;
    let _ = daemon::recover_owned_state(&conn, &id.instance_id, id.generation)?;
    let server = AgentGraphServer::new(
        "http://127.0.0.1:11434".into(),
        "glm-5.2:cloud".into(),
        Some(data.clone()),
        std::env::var_os("AGENT_GRAPH_INTEGRITY_KEY_PATH").map(PathBuf::from),
    )
    .map_err(std::io::Error::other)?;
    if socket.exists() {
        std::fs::remove_file(&socket)?;
    }
    if let Some(p) = socket.parent() {
        std::fs::create_dir_all(p)?;
    }
    let listener = UnixListener::bind(&socket)?;
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600))?;
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let server = std::sync::Arc::new(server);
        for stream in listener.incoming() {
            let stream = stream?;
            let server = server.clone();
            std::thread::spawn(move || {
                // Each connection gets the daemon-owned handler state. The
                // listener, lock, recovery, and durable store are initialized
                // exactly once above; rmcp's per-session service is isolated.
                let _ = serve_connection(stream, server);
            });
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;
    Ok(())
}

fn serve_connection(
    stream: UnixStream,
    server: std::sync::Arc<AgentGraphServer>,
) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async move {
        let socket = tokio::net::UnixStream::from_std(stream)?;
        let (mut socket_reader, mut socket_writer) = socket.into_split();
        let (rmcp_input, rmcp_transport) = tokio::io::duplex(transport::MAX_FRAME + 4096);
        let (rmcp_reader, mut rmcp_writer) = tokio::io::split(rmcp_input);

        let socket_to_rmcp = async {
            loop {
                let mut header = [0u8; 4];
                socket_reader.read_exact(&mut header).await?;
                let length = u32::from_be_bytes(header) as usize;
                if length > transport::MAX_FRAME {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "frame too large",
                    ));
                }
                let mut frame = vec![0u8; length];
                socket_reader.read_exact(&mut frame).await?;
                rmcp_writer.write_all(&frame).await?;
                rmcp_writer.write_all(b"\n").await?;
            }
            #[allow(unreachable_code)]
            Ok::<(), std::io::Error>(())
        };

        let rmcp_to_socket = async {
            let mut lines = tokio::io::BufReader::new(rmcp_reader).lines();
            while let Some(line) = lines.next_line().await? {
                transport::write_frame_async(&mut socket_writer, line.as_bytes()).await?;
            }
            Ok::<(), std::io::Error>(())
        };

        let service = server
            .serve(rmcp_transport)
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        let bridges = tokio::try_join!(socket_to_rmcp, rmcp_to_socket);
        let _ = service.cancel().await;
        bridges.map(|_| ()).map_err(Into::into)
    })
}
