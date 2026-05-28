use forge_pilot::{
    answer_repo_question, provider_fallback_allowed, render_terminal_answer,
    repo_chat_provider_grounding_message,
};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{self, Write};

use super::storage::open_resources;
use super::tui::AppState;
use super::{prompt, prompt_default};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub(super) enum ChatProviderConfig {
    Ollama {
        base_url: String,
        model: String,
    },
    OpenAi {
        base_url: String,
        model: String,
        api_key: String,
    },
}

pub(super) async fn configure_provider(state: &mut AppState) -> Result<(), String> {
    println!("Provider setup");
    println!("Provider fallback is used only for clearly non-repo questions.");
    println!("Provider responses stream in the chat view when supported.");
    println!("1. Ollama");
    println!("2. OpenAI");
    println!("3. Disable chat provider");

    match prompt("Choose a provider")?.trim() {
        "1" => {
            let base_url = prompt_default("Ollama base URL", state.ollama_base_url())?;
            let model = prompt_default("Ollama model", state.ollama_model())?;
            state.provider = Some(ChatProviderConfig::Ollama { base_url, model });
            println!("Ollama is configured.");
        }
        "2" => {
            let base_url = prompt_default("OpenAI base URL", state.openai_base_url())?;
            let model = prompt_default("OpenAI model", state.openai_model())?;
            let api_key =
                prompt_default("OpenAI API key", state.openai_api_key().unwrap_or_default())?;
            state.provider = Some(ChatProviderConfig::OpenAi {
                base_url,
                model,
                api_key,
            });
            println!("OpenAI is configured.");
        }
        "3" => {
            state.provider = None;
            println!("Chat provider disabled.");
        }
        _ => println!("No provider change applied."),
    }

    Ok(())
}

pub(super) async fn chat_loop(state: &AppState) -> Result<(), String> {
    println!("Plain-English chat");
    println!("Press enter on an empty line to leave chat.");
    println!(
        "Repo-grounded answers render locally; provider fallback streams for non-repo questions."
    );
    let config = state.to_loop_config();
    let resources = open_resources(&state.memory_dir, &state.forge_db, &config)?;
    let mut history = vec![ChatMessage {
        role: "system".into(),
        content: "You are Forge Pilot's plain-English terminal assistant. Use grounded workspace-source citations when available and abstain when the source memory cannot support a claim.".into(),
    }];

    loop {
        let input = prompt("You")?;
        if input.trim().is_empty() {
            break;
        }
        history.push(ChatMessage {
            role: "user".into(),
            content: input.trim().into(),
        });
        let grounded = answer_repo_question(
            &resources.runtime,
            &resources.memory_store,
            &config,
            input.trim(),
        )
        .await
        .map_err(|error| error.to_string())?;
        let allow_fallback =
            !grounded.grounded && state.provider.is_some() && provider_fallback_allowed(&grounded);
        let reply = if allow_fallback {
            let provider = match state.provider.as_ref() {
                Some(provider) => provider,
                // LIB-HIGH-001: replaced unreachable!() with recoverable error
                None => return Err("provider not available after validation".into()),
            };
            let augmented_history = augment_history_with_grounding(&history, &grounded);
            print!("Pilot: ");
            io::stdout().flush().map_err(|error| error.to_string())?;
            chat_with_provider(provider, &augmented_history).await?
        } else {
            render_terminal_answer(&grounded)
        };
        if !allow_fallback {
            println!("Pilot: {reply}");
        } else {
            println!();
        }
        history.push(ChatMessage {
            role: "assistant".into(),
            content: reply,
        });
    }

    Ok(())
}

pub(super) async fn chat_with_provider(
    provider: &ChatProviderConfig,
    history: &[ChatMessage],
) -> Result<String, String> {
    match provider {
        ChatProviderConfig::Ollama { base_url, model } => {
            chat_with_ollama(base_url, model, history).await
        }
        ChatProviderConfig::OpenAi {
            base_url,
            model,
            api_key,
        } => chat_with_openai(base_url, model, api_key, history).await,
    }
}

fn augment_history_with_grounding(
    history: &[ChatMessage],
    grounded: &forge_pilot::RepoChatAnswer,
) -> Vec<ChatMessage> {
    let mut augmented = history.to_vec();
    augmented.push(ChatMessage {
        role: "system".into(),
        content: format!(
            "Answer from grounded imported repo memory first. Do not contradict the grounding. If grounding is unavailable, say so plainly.\n{}",
            repo_chat_provider_grounding_message(grounded)
        ),
    });
    augmented
}

async fn chat_with_ollama(
    base_url: &str,
    model: &str,
    history: &[ChatMessage],
) -> Result<String, String> {
    // LIB-HIGH-002: add HTTP timeouts to prevent indefinite hangs
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("failed to build HTTP client: {e}"))?;
    let mut response = client
        .post(format!("{}/api/chat", trim_trailing_slash(base_url)))
        .json(&json!({
            "model": model,
            "messages": history,
            "stream": true
        }))
        .send()
        .await
        .map_err(|error| format!("ollama request failed: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        let value: serde_json::Value = response
            .json()
            .await
            .map_err(|error| format!("ollama response decode failed: {error}"))?;
        return Err(format!("ollama returned {status}: {value}"));
    }

    stream_json_lines(&mut response, "ollama response stream", |value| {
        value
            .get("message")
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .map(ToString::to_string)
    })
    .await
}

async fn chat_with_openai(
    base_url: &str,
    model: &str,
    api_key: &str,
    history: &[ChatMessage],
) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err("OpenAI API key is not configured.".into());
    }

    // LIB-HIGH-002: add HTTP timeouts to prevent indefinite hangs
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("failed to build HTTP client: {e}"))?;
    let mut response = client
        .post(format!(
            "{}/v1/chat/completions",
            trim_trailing_slash(base_url)
        ))
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header(CONTENT_TYPE, "application/json")
        .json(&json!({
            "model": model,
            "messages": history,
            "stream": true
        }))
        .send()
        .await
        .map_err(|error| format!("openai request failed: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        let value: serde_json::Value = response
            .json()
            .await
            .map_err(|error| format!("openai response decode failed: {error}"))?;
        return Err(format!("openai returned {status}: {value}"));
    }

    stream_sse_lines(&mut response, "openai response stream", |value| {
        value
            .get("choices")
            .and_then(|choices| choices.as_array())
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("delta"))
            .and_then(|delta| delta.get("content"))
            .and_then(|content| content.as_str())
            .map(ToString::to_string)
    })
    .await
}

async fn stream_json_lines<F>(
    response: &mut reqwest::Response,
    stream_name: &str,
    mut extract: F,
) -> Result<String, String>
where
    F: FnMut(&serde_json::Value) -> Option<String>,
{
    let mut buffer = String::new();
    let mut reply = String::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| format!("{stream_name} read failed: {error}"))?
    {
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(index) = buffer.find('\n') {
            let line = buffer[..index].trim().to_string();
            buffer.drain(..=index);
            if line.is_empty() {
                continue;
            }
            let value: serde_json::Value = serde_json::from_str(&line)
                .map_err(|error| format!("{stream_name} decode failed: {error}; line={line}"))?;
            if let Some(content) = extract(&value) {
                print!("{content}");
                io::stdout().flush().map_err(|error| error.to_string())?;
                reply.push_str(&content);
            }
        }
    }
    if !buffer.trim().is_empty() {
        let value: serde_json::Value = serde_json::from_str(buffer.trim())
            .map_err(|error| format!("{stream_name} decode failed: {error}; trailing={buffer}"))?;
        if let Some(content) = extract(&value) {
            print!("{content}");
            io::stdout().flush().map_err(|error| error.to_string())?;
            reply.push_str(&content);
        }
    }
    let reply = reply.trim().to_string();
    if reply.is_empty() {
        Err(format!(
            "{stream_name} did not include any assistant content"
        ))
    } else {
        Ok(reply)
    }
}

async fn stream_sse_lines<F>(
    response: &mut reqwest::Response,
    stream_name: &str,
    mut extract: F,
) -> Result<String, String>
where
    F: FnMut(&serde_json::Value) -> Option<String>,
{
    let mut buffer = String::new();
    let mut reply = String::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| format!("{stream_name} read failed: {error}"))?
    {
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(index) = buffer.find('\n') {
            let line = buffer[..index].trim().to_string();
            buffer.drain(..=index);
            if line.is_empty() || !line.starts_with("data:") {
                continue;
            }
            let payload = line.trim_start_matches("data:").trim();
            if payload == "[DONE]" {
                let reply = reply.trim().to_string();
                return if reply.is_empty() {
                    Err(format!("{stream_name} ended without assistant content"))
                } else {
                    Ok(reply)
                };
            }
            let value: serde_json::Value = serde_json::from_str(payload).map_err(|error| {
                format!("{stream_name} decode failed: {error}; payload={payload}")
            })?;
            if let Some(content) = extract(&value) {
                print!("{content}");
                io::stdout().flush().map_err(|error| error.to_string())?;
                reply.push_str(&content);
            }
        }
    }
    let reply = reply.trim().to_string();
    if reply.is_empty() {
        Err(format!(
            "{stream_name} did not include any assistant content"
        ))
    } else {
        Ok(reply)
    }
}

fn trim_trailing_slash(value: &str) -> &str {
    value.trim_end_matches('/')
}
