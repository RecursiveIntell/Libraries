//! Linux fault witness: cancelling the host future must stop owned descendants.
//! Test-only cleanup kills only the fixed fixture's still-owned process group.
#![cfg(target_os = "linux")]
use check_runner::{BackendConfig, ExecutionBackend, HostBackend};
use std::time::Duration;

fn live(pid: u32) -> bool {
    std::fs::read_to_string(format!("/proc/{pid}/stat"))
        .ok()
        .and_then(|text| {
            text.rsplit_once(") ")
                .map(|(_, rest)| rest.starts_with('Z'))
        })
        .is_some_and(|zombie| !zombie)
}

#[test]
fn cancellation_stops_owned_host_descendant() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let ready = path.join("ready");
        let backend = HostBackend::new(&BackendConfig {
            mode: "standard".into(), execution_backend_preference: "host".into(),
            container_runtime_preference: "auto".into(), sealed_allow_host_backend: false,
            rust_image: "unused".into(), command_timeout_secs: 20,
            memory_limit: "256m".into(), cpu_limit: "1".into(),
        });
        let task = tokio::spawn(async move {
            backend.run_command(&path, "sh", &["-c", "sleep 30 & child=$!; printf '%s %s' $$ $child > ready; wait"], &[], 20).await
        });
        let text = tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                if let Ok(text) = std::fs::read_to_string(&ready) {
                    if text.split_whitespace().count() == 2 { break text; }
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        }).await.unwrap();
        let pids: Vec<u32> = text.split_whitespace().map(|value| value.parse().unwrap()).collect();
        let leader = pids[0];
        let descendant = pids[1];
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        let deadline = tokio::time::Instant::now() + Duration::from_secs(1);
        while live(descendant) && tokio::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        let escaped = live(descendant);
        // Preserve failure honesty but do not leave the fixture's 30s sleeper.
        if escaped {
            let stat = std::fs::read_to_string(format!("/proc/{descendant}/stat")).unwrap();
            let rest = stat.rsplit_once(") ").unwrap().1;
            let pgrp: u32 = rest.split_whitespace().nth(2).unwrap().parse().unwrap();
            assert_eq!(pgrp, leader, "refuse cleanup of an unexpected process group");
            check_runner_sys::kill_process_group(leader as i32);
        }
        assert!(!escaped, "cancelled HostBackend left its owned descendant alive (cleanup executed); host is not a qualified campaign boundary");
    });
}
