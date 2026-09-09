use agent_collaboration_transport::{
    config::PeerConfigV1, identity::StaticIdentity, read_frame, send_authenticated_frame,
    ReplayCache,
};
use chrono::{Duration, Utc};
use std::path::Path;
use tokio::net::TcpListener;

fn read_key(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if !metadata.file_type().is_file() {
        return Err("key path must be a regular file".into());
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o077 != 0 {
        return Err("key file permissions must exclude group and other access".into());
    }
    let key = std::fs::read(path).map_err(|error| error.to_string())?;
    if key.len() < 32 {
        return Err("key file must contain at least 32 bytes".into());
    }
    Ok(key)
}

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn parse_config_arg() -> Result<String, String> {
    let mut args = std::env::args().skip(1);
    match (args.next().as_deref(), args.next(), args.next()) {
        (Some("--config"), Some(path), None) => Ok(path),
        _ => Err("usage: agent-collaboration-worker --config PATH".into()),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    let config_path = parse_config_arg()?;
    let config =
        PeerConfigV1::from_path(Path::new(&config_path)).map_err(|error| error.to_string())?;
    let identity = StaticIdentity::new(
        config.agent_id.clone(),
        read_key(Path::new(&config.key_file))?,
    )
    .map_err(|error| error.to_string())?;
    let (_, peer) = config
        .peers
        .iter()
        .next()
        .ok_or_else(|| "at least one pinned peer is required".to_owned())?;
    if config.peers.len() != 1 {
        return Err("worker V1 requires exactly one pinned peer".into());
    }
    let peer_identity =
        StaticIdentity::new(peer.agent_id.clone(), read_key(Path::new(&peer.key_file))?)
            .map_err(|error| error.to_string())?;
    let listener = TcpListener::bind(&config.bind_addr)
        .await
        .map_err(|error| error.to_string())?;
    let replay = ReplayCache::new();
    loop {
        let (mut stream, _) = listener.accept().await.map_err(|error| error.to_string())?;
        let result = async {
            let frame = read_frame(&mut stream)
                .await
                .map_err(|error| error.to_string())?;
            identity
                .verify_frame(
                    &frame,
                    &peer_identity,
                    &config.capability_manifest_digest,
                    Utc::now(),
                )
                .map_err(|error| error.to_string())?;
            if !replay.admit(&frame.nonce) {
                return Err("replayed nonce".to_owned());
            }
            let response = serde_json::to_vec(&serde_json::json!({
                "ok": true,
                "status": "received_non_effectful_frame",
                "payload_bytes": frame.payload.len(),
                "capability_manifest_digest": config.capability_manifest_digest
            }))
            .map_err(|error| error.to_string())?;
            let response_frame = identity
                .issue_frame(
                    peer_identity.agent_id.clone(),
                    config.capability_manifest_digest.clone(),
                    (Utc::now() + Duration::minutes(5)).to_rfc3339(),
                    response,
                )
                .map_err(|error| error.to_string())?;
            send_authenticated_frame(&mut stream, &response_frame)
                .await
                .map_err(|error| error.to_string())?;
            Ok::<(), String>(())
        }
        .await;
        if let Err(error) = result {
            eprintln!("collaboration connection rejected: {error}");
        }
    }
}
