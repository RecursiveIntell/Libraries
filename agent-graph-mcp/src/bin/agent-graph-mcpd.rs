use agent_graph_mcp::{daemon, AgentGraphServer};
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
    drop(conn);
    if socket.exists() {
        std::fs::remove_file(&socket)?;
    }
    if let Some(p) = socket.parent() {
        std::fs::create_dir_all(p)?;
    }
    let listener = UnixListener::bind(&socket)?;
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600))?;
    let rt = tokio::runtime::Runtime::new()?;
    let data_dir = data.clone();
    let key_path = std::env::var_os("AGENT_GRAPH_INTEGRITY_KEY_PATH").map(PathBuf::from);
    rt.block_on(async move {
        for stream in listener.incoming() {
            let stream = stream?;
            let data_dir = data_dir.clone();
            let key_path = key_path.clone();
            tokio::task::spawn_blocking(move || {
                let _ = serve_connection(stream, &data_dir, key_path.as_deref());
            });
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;
    Ok(())
}

fn serve_connection(
    stream: UnixStream,
    data_dir: &std::path::Path,
    key_path: Option<&std::path::Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async move {
        let socket = tokio::net::UnixStream::from_std(stream)?;
        let (mut sock_rx, mut sock_tx) = socket.into_split();

        // Create a duplex: bridge_side <-> rmcp_side
        // rmcp reads JSON-RPC lines from rmcp_side and writes responses as lines to rmcp_side
        // We bridge: decode frames -> write lines to bridge -> rmcp reads -> rmcp writes response lines -> we read and encode as frames
        let (bridge_side, rmcp_side) = tokio::io::duplex(1024 * 1024 + 4096);
        let (bridge_rx, mut bridge_tx) = tokio::io::split(bridge_side);

        // Bridge: socket frames -> rmcp input (as newline-delimited JSON)
        let to_rmcp = async {
            loop {
                let mut hdr = [0u8; 4];
                if sock_rx.read_exact(&mut hdr).await.is_err() {
                    break;
                };
                let len = u32::from_be_bytes(hdr) as usize;
                if len > 1024 * 1024 {
                    break;
                }
                let mut payload = vec![0u8; len];
                if sock_rx.read_exact(&mut payload).await.is_err() {
                    break;
                }
                if bridge_tx.write_all(&payload).await.is_err() {
                    break;
                }
                if bridge_tx.write_all(b"\n").await.is_err() {
                    break;
                }
                if bridge_tx.flush().await.is_err() {
                    break;
                }
            }
            // Signal EOF to rmcp by closing write side
            drop(bridge_tx);
        };

        // Bridge: rmcp output -> socket frames
        let from_rmcp = async {
            let mut reader = tokio::io::BufReader::new(bridge_rx);
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let trimmed = line.trim_end();
                        if trimmed.is_empty() {
                            continue;
                        }
                        let len = trimmed.len() as u32;
                        if sock_tx.write_all(&len.to_be_bytes()).await.is_err() {
                            break;
                        }
                        if sock_tx.write_all(trimmed.as_bytes()).await.is_err() {
                            break;
                        }
                        if sock_tx.flush().await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        };

        // Create server and serve on rmcp_side of the duplex
        let server = AgentGraphServer::new(
            "http://127.0.0.1:11434".into(),
            "glm-5.2:cloud".into(),
            Some(data_dir.to_path_buf()),
            key_path.map(|p| p.to_path_buf()),
        )
        .map_err(std::io::Error::other)?;

        // Use serve() which handles the initialize handshake
        let serve = async {
            let service = match server.serve(rmcp_side).await {
                Ok(s) => s,
                Err(_) => return,
            };
            let _ = service.cancel().await;
        };

        tokio::select! {
            _ = serve => {}
            _ = to_rmcp => {}
            _ = from_rmcp => {}
        }

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
