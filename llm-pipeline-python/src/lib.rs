#![allow(deprecated)]

//! Python bindings for llm-pipeline — Hermes transport replacement.
//!
//! ``Pipeline`` wraps ``LlmCall`` + ``ExecCtx`` so Hermes can route LLM
//! requests through the Rust pipeline instead of raw Python httpx calls.
//! Uses pyo3-async-runtimes for safe tokio bridging.

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

// ─── LlmConfig ───────────────────────────────────────────────────

#[pyclass(name = "LlmConfig")]
#[derive(Clone)]
pub struct PyLlmConfig {
    pub temperature: f64,
    pub max_tokens: u32,
    pub thinking: bool,
    pub json_mode: bool,
}

#[pymethods]
impl PyLlmConfig {
    #[new]
    #[pyo3(signature = (temperature = 0.7, max_tokens = 2048, thinking = false, json_mode = false))]
    fn new(temperature: f64, max_tokens: u32, thinking: bool, json_mode: bool) -> Self {
        Self {
            temperature,
            max_tokens,
            thinking,
            json_mode,
        }
    }

    fn with_temperature(&self, temp: f64) -> Self {
        Self {
            temperature: temp,
            ..self.clone()
        }
    }

    fn with_max_tokens(&self, tokens: u32) -> Self {
        Self {
            max_tokens: tokens,
            ..self.clone()
        }
    }

    fn with_thinking(&self, enabled: bool) -> Self {
        Self {
            thinking: enabled,
            ..self.clone()
        }
    }

    fn with_json_mode(&self, enabled: bool) -> Self {
        Self {
            json_mode: enabled,
            ..self.clone()
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "LlmConfig(temp={:.1}, max_tokens={}, thinking={}, json={})",
            self.temperature, self.max_tokens, self.thinking, self.json_mode
        )
    }
}

impl PyLlmConfig {
    fn to_rust(&self) -> llm_pipeline::LlmConfig {
        llm_pipeline::LlmConfig {
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            thinking: self.thinking,
            json_mode: self.json_mode,
            options: None,
            constraint: llm_pipeline::constraints::GenerationConstraint::None,
        }
    }
}

// ─── Pipeline ─────────────────────────────────────────────────────

#[pyclass(name = "Pipeline")]
pub struct PyPipeline {
    url: String,
    model: String,
    default_config: PyLlmConfig,
}

#[pymethods]
impl PyPipeline {
    #[new]
    #[pyo3(signature = (url, model, *, config = None))]
    fn new(url: &str, model: &str, config: Option<PyLlmConfig>) -> Self {
        Self {
            url: url.to_string(),
            model: model.to_string(),
            default_config: config.unwrap_or(PyLlmConfig {
                temperature: 0.7,
                max_tokens: 2048,
                thinking: false,
                json_mode: false,
            }),
        }
    }

    fn __repr__(&self) -> String {
        format!("Pipeline(url={}, model={})", self.url, self.model)
    }

    /// Call the LLM with a prompt, returning raw response text.
    #[pyo3(signature = (prompt, *, system = None, config = None))]
    fn call(
        &self,
        prompt: &str,
        system: Option<&str>,
        config: Option<PyLlmConfig>,
    ) -> PyResult<String> {
        use llm_pipeline::payload::Payload;
        let cfg = config
            .as_ref()
            .map(|c| c.to_rust())
            .unwrap_or_else(|| self.default_config.to_rust());
        let ctx = llm_pipeline::ExecCtx::builder(&self.url).build();
        let call = llm_pipeline::LlmCall::new("py", prompt)
            .with_model(&self.model)
            .with_config(cfg);
        let call = if let Some(sys) = system {
            call.with_system(sys)
        } else {
            call
        };
        let input = serde_json::Value::String(prompt.to_string());
        let out = pyo3_async_runtimes::tokio::get_runtime()
            .block_on(call.invoke(&ctx, input))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(out.raw_response)
    }

    /// Call with JSON schema constraint for structured output.
    #[pyo3(signature = (prompt, json_schema, *, system = None, config = None))]
    fn call_structured(
        &self,
        prompt: &str,
        json_schema: &str,
        system: Option<&str>,
        config: Option<PyLlmConfig>,
    ) -> PyResult<String> {
        use llm_pipeline::payload::Payload;
        let schema: serde_json::Value = serde_json::from_str(json_schema)
            .map_err(|e| PyRuntimeError::new_err(format!("invalid JSON schema: {e}")))?;
        let mut cfg = config
            .as_ref()
            .map(|c| c.to_rust())
            .unwrap_or_else(|| self.default_config.to_rust());
        cfg = cfg.with_json_schema(schema);
        let ctx = llm_pipeline::ExecCtx::builder(&self.url).build();
        let call = llm_pipeline::LlmCall::new("py-structured", prompt)
            .with_model(&self.model)
            .with_config(cfg);
        let call = if let Some(sys) = system {
            call.with_system(sys)
        } else {
            call
        };
        let input = serde_json::Value::String(prompt.to_string());
        let out = pyo3_async_runtimes::tokio::get_runtime()
            .block_on(call.invoke(&ctx, input))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(out.raw_response)
    }
}

// ─── Module ───────────────────────────────────────────────────────

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyLlmConfig>()?;
    m.add_class::<PyPipeline>()?;
    Ok(())
}
