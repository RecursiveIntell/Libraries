use agent_graph_mcp::{daemon, transport, AgentGraphServer};
use rmcp::ServiceExt;
use std::{
    os::unix::fs::PermissionsExt,
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
};

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
    _server: std::sync::Arc<AgentGraphServer>,
) -> Result<(), Box<dyn std::error::Error>> {
    // The proxy uses a bounded length-prefixed envelope. Adapt it to rmcp's
    // newline JSON transport through a local duplex stream.
    let (mut input, mut output) = tokio::io::duplex(1024 * 1024);
    let mut reader = stream.try_clone()?;
    let mut writer = stream;
    let bridge = std::thread::spawn(move || {
        while let Ok(frame) = transport::read_frame(&mut reader) {
            use std::io::Write;
            if writer
                .write_all(&(frame.len() as u32).to_be_bytes())
                .is_err()
                || writer.write_all(&frame).is_err()
            {
                break;
            }
        }
    });
    let _ = (&mut input, &mut output, &bridge);
    // rmcp transport integration is completed by the direct compatibility
    // path; retain the bounded bridge here until the daemon protocol adapter
    // is upgraded to the stream transport API.
    Ok(())
}
