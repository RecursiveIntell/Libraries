use std::io::{self, BufRead, Write};

use agent_graph_mcp::protocol::{RpcError, RpcRequest, RpcResponse};
use agent_graph_mcp::Server;

fn main() {
    let mut base_url = "http://127.0.0.1:11434".to_string();
    let mut default_model = "glm-5.2:cloud".to_string();
    let args: Vec<String> = std::env::args().collect();
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--base-url" => {
                if let Some(value) = iter.next() {
                    base_url = value.clone();
                }
            }
            "--model" => {
                if let Some(value) = iter.next() {
                    default_model = value.clone();
                }
            }
            "--help" => {
                eprintln!("agent-graph-mcp [--base-url server-admin-url] [--model server-alias]");
                return;
            }
            _ => {}
        }
    }
    let mut server = Server::new(base_url, default_model);
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<RpcRequest>(&line) {
            Ok(request) => server.handle_request(&request),
            Err(error) => RpcResponse {
                jsonrpc: "2.0".into(),
                id: serde_json::Value::Null,
                result: None,
                error: Some(RpcError {
                    code: -32700,
                    message: format!("parse error: {error}"),
                }),
            },
        };
        let encoded = serde_json::to_string(&response).unwrap_or_else(|_| {
            r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"serialize error"}}"#
                .into()
        });
        if writeln!(stdout, "{encoded}").is_err() {
            break;
        }
        let _ = stdout.flush();
    }
}
