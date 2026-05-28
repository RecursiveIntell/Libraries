use crate::parser::{self, ParseError};
use crate::types::{OllamaVisionConfig, TagOptions};
use reqwest::Client;
use serde_json::json;
use std::path::Path;

const DEFAULT_TAG_PROMPT: &str = r#"You are an image tagging assistant. Analyze the provided image and return a JSON array of relevant tags. Each tag should be a single word or short phrase (2-3 words max) that describes a key visual element, style, subject, or mood in the image.

Return ONLY a JSON array of strings. Example: ["portrait", "fantasy", "dark lighting", "woman", "medieval", "oil painting"]

Return between 5 and 15 tags. Focus on:
- Subject matter (person, animal, landscape, object)
- Art style (photorealistic, anime, oil painting, digital art)
- Mood/atmosphere (dark, bright, serene, dramatic)
- Colors (warm tones, blue, monochrome)
- Composition (close-up, wide shot, symmetrical)
- Notable elements (fire, water, armor, flowers)"#;

/// Tag an image using an Ollama vision model.
///
/// Returns a list of cleaned, lowercase tag strings extracted from the
/// model's response using the 7-strategy parser.
///
/// # Errors
///
/// Returns an error if:
/// - The image file cannot be read
/// - The Ollama endpoint is unreachable
/// - The model returns an error status
/// - The response cannot be parsed into tags
pub async fn tag_image(
    client: &Client,
    config: &OllamaVisionConfig,
    image_path: &Path,
    options: &TagOptions,
) -> Result<Vec<String>, TagError> {
    let image_b64 = read_image_base64(image_path)?;

    let prompt = options.prompt.as_deref().unwrap_or(DEFAULT_TAG_PROMPT);

    let mut body = json!({
        "model": config.model,
        "prompt": prompt,
        "images": [image_b64],
        "stream": false,
        "options": config.options,
    });

    if options.request_json_format {
        body["format"] = json!("json");
    }

    let url = format!("{}/api/generate", config.endpoint);
    let resp = client
        .post(&url)
        .timeout(config.timeout)
        .json(&body)
        .send()
        .await
        .map_err(|e| TagError::Connection(config.endpoint.clone(), e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        return Err(TagError::OllamaError(status, text));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| TagError::InvalidResponse(e.to_string()))?;

    let content = json
        .get("response")
        .and_then(|v| v.as_str())
        .unwrap_or("[]");

    let tags = parser::parse_tags(content).map_err(TagError::Parse)?;
    Ok(truncate_tags(
        tags,
        options.max_tags,
        options.max_tag_length,
    ))
}

/// Tag an image from raw base64-encoded bytes (no file I/O).
///
/// Useful when you already have the image in memory.
pub async fn tag_image_base64(
    client: &Client,
    config: &OllamaVisionConfig,
    image_b64: &str,
    options: &TagOptions,
) -> Result<Vec<String>, TagError> {
    let prompt = options.prompt.as_deref().unwrap_or(DEFAULT_TAG_PROMPT);

    let mut body = json!({
        "model": config.model,
        "prompt": prompt,
        "images": [image_b64],
        "stream": false,
        "options": config.options,
    });

    if options.request_json_format {
        body["format"] = json!("json");
    }

    let url = format!("{}/api/generate", config.endpoint);
    let resp = client
        .post(&url)
        .timeout(config.timeout)
        .json(&body)
        .send()
        .await
        .map_err(|e| TagError::Connection(config.endpoint.clone(), e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        return Err(TagError::OllamaError(status, text));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| TagError::InvalidResponse(e.to_string()))?;

    let content = json
        .get("response")
        .and_then(|v| v.as_str())
        .unwrap_or("[]");

    let tags = parser::parse_tags(content).map_err(TagError::Parse)?;
    Ok(truncate_tags(
        tags,
        options.max_tags,
        options.max_tag_length,
    ))
}

/// Apply tag count and length limits.
fn truncate_tags(tags: Vec<String>, max_tags: usize, max_tag_length: usize) -> Vec<String> {
    tags.into_iter()
        .take(max_tags)
        .map(|t| truncate_utf8(&t, max_tag_length))
        .collect()
}

fn truncate_utf8(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    text.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::truncate_utf8;

    #[test]
    fn truncate_utf8_preserves_codepoint_boundaries() {
        assert_eq!(truncate_utf8("naive", 3), "nai");
        assert_eq!(truncate_utf8("cafe\u{301}", 4), "cafe");
        assert_eq!(truncate_utf8("こんにちは", 3), "こんに");
    }
}

/// Errors that can occur during image tagging.
#[derive(Debug, thiserror::Error)]
pub enum TagError {
    #[error("Cannot connect to Ollama at {0}: {1}")]
    Connection(String, String),

    #[error("Ollama returned HTTP {0}: {1}")]
    OllamaError(u16, String),

    #[error("Invalid response from Ollama: {0}")]
    InvalidResponse(String),

    #[error("Failed to read image: {0}")]
    ImageRead(String),

    #[error("{0}")]
    Parse(#[from] ParseError),
}

impl TagError {
    /// Stable string discriminant for programmatic matching.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Connection(..) => "connection",
            Self::OllamaError(..) => "ollama_error",
            Self::InvalidResponse(_) => "invalid_response",
            Self::ImageRead(_) => "image_read",
            Self::Parse(_) => "parse",
        }
    }
}

fn read_image_base64(path: &Path) -> Result<String, TagError> {
    let bytes = std::fs::read(path)
        .map_err(|e| TagError::ImageRead(format!("{}: {}", path.display(), e)))?;
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &bytes,
    ))
}
