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

        // Create a duplex: rmcp_transport is given to serve(), rmcp_input is our bridge end.
        // rmcp reads JSON-RPC lines from one side and writes responses as lines to the other.
        let (rmcp_bridge, rmcp_transport) = tokio::io::duplex(transport::MAX_FRAME + 4096);
        let (bridge_reader, mut bridge_writer) = tokio::io::split(rmcp_bridge);

        // Task 1: socket frames → bridge writer (write frame body + newline as JSON-RPC line)
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
                bridge_writer.write_all(&frame).await?;
                bridge_writer.write_all(b"\n").await?;
                bridge_writer.flush().await?;
            }
            #[allow(unreachable_code)]
            Ok::<(), std::io::Error>(())
        };

        // Task 2: bridge reader lines → socket frames (read JSON-RPC response lines, write as frames)
        let rmcp_to_socket = async {
            let mut lines = tokio::io::BufReader::new(bridge_reader).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if line.is_empty() {
                    continue;
                }
                transport::write_frame_async(&mut socket_writer, line.as_bytes()).await?;
                socket_writer.flush().await?;
            }
            Ok::<(), std::io::Error>(())
        };

        // Task 3: rmcp serve on the transport side of the duplex
        let serve = async {
            let service = server
                .serve(rmcp_transport)
                .await
                .map_err(|e| std::io::Error::other(e.to_string()))?;
            let _ = service.cancel().await;
            Ok::<(), std::io::Error>(())
        };

        // Run all three concurrently. When the socket closes, the bridges end,
        // and serve will complete when its transport drops.
        tokio::select! {
            result = serve => {
                let _ = result;
            }
            result = socket_to_rmcp => {
                let _ = result;
            }
            result = rmcp_to_socket => {
                let _ = result;
            }
        }

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
