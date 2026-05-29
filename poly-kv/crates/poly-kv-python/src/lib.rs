use poly_kv::{
    CompressionPolicyV1, DType, ExactKvBlock, KvAttentionKind, KvCacheShapeV2, KvLayout, KvRole,
    KvSliceRequest, KvTensorShape, LayerId, ModelFingerprint, PolyKvError, Q8KeyCodec,
    RawExactValueCodec, ReaderConfig, SharedKvPool, TokenSpan, TokenizerFingerprint,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
struct ShapeSpec {
    batch: u32,
    layers: u32,
    num_q_heads: u32,
    num_kv_heads: u32,
    seq_len: u64,
    head_dim: u32,
    layout: String,
    dtype: String,
    attention_kind: String,
}

#[derive(Debug, Deserialize)]
struct BlockSpec {
    role: String,
    layer: u32,
    data: Vec<f32>,
}

#[pyfunction]
fn validate_shape_json(shape_json: &str) -> PyResult<String> {
    let spec = parse_shape(shape_json)?;
    let _ = shape_v2(&spec)?;
    Ok(json!({
        "ok": true,
        "batch": spec.batch,
        "layers": spec.layers,
        "num_q_heads": spec.num_q_heads,
        "num_kv_heads": spec.num_kv_heads,
        "seq_len": spec.seq_len,
        "head_dim": spec.head_dim,
        "layout": spec.layout,
        "dtype": spec.dtype,
        "attention_kind": spec.attention_kind,
    })
    .to_string())
}

#[pyfunction]
fn build_synthetic_pool_receipts_json(shape_json: &str) -> PyResult<String> {
    let shape = legacy_shape_from_json(shape_json)?;
    let blocks = synthetic_blocks(&shape)?;
    let pool = build_pool(shape, blocks)?;
    let reader = pool
        .attach_reader(ReaderConfig::default())
        .map_err(py_poly_err)?;
    Ok(json!({
        "manifest": pool.manifest(),
        "build_receipt": pool.build_receipt(),
        "reader_receipt": reader.injection_receipt(),
    })
    .to_string())
}

#[pyfunction]
fn attach_synthetic_reader_receipt_json(shape_json: &str) -> PyResult<String> {
    let shape = legacy_shape_from_json(shape_json)?;
    let blocks = synthetic_blocks(&shape)?;
    let pool = build_pool(shape, blocks)?;
    let reader = pool
        .attach_reader(ReaderConfig::default())
        .map_err(py_poly_err)?;
    serde_json::to_string(reader.injection_receipt()).map_err(py_json_err)
}

#[pyfunction]
fn decode_synthetic_slice_receipt_json(
    shape_json: &str,
    role: &str,
    layer: u32,
    start: u64,
    end: u64,
) -> PyResult<String> {
    let shape = legacy_shape_from_json(shape_json)?;
    let blocks = synthetic_blocks(&shape)?;
    let pool = build_pool(shape, blocks)?;
    let reader = pool
        .attach_reader(ReaderConfig::default())
        .map_err(py_poly_err)?;
    let req = KvSliceRequest::layer_span(
        LayerId(layer),
        TokenSpan::new(start, end).map_err(py_shape_err)?,
    )
    .for_role(parse_role(role)?);
    let decoded = reader.decode_slice(req).map_err(py_poly_err)?;
    Ok(json!({
        "data_len": decoded.data.len(),
        "receipt": decoded.receipt,
    })
    .to_string())
}

#[pyfunction]
fn build_pool_from_f32_json(shape_json: &str, blocks_json: &str) -> PyResult<String> {
    let shape = legacy_shape_from_json(shape_json)?;
    let specs = serde_json::from_str::<Vec<BlockSpec>>(blocks_json).map_err(py_json_err)?;
    let mut blocks = Vec::with_capacity(specs.len());
    for spec in specs {
        blocks.push(
            ExactKvBlock::new(
                parse_role(&spec.role)?,
                LayerId(spec.layer),
                shape.clone(),
                spec.data,
            )
            .map_err(py_poly_err)?,
        );
    }
    let pool = build_pool(shape, blocks)?;
    Ok(json!({
        "manifest": pool.manifest(),
        "build_receipt": pool.build_receipt(),
    })
    .to_string())
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(validate_shape_json, m)?)?;
    m.add_function(wrap_pyfunction!(build_synthetic_pool_receipts_json, m)?)?;
    m.add_function(wrap_pyfunction!(attach_synthetic_reader_receipt_json, m)?)?;
    m.add_function(wrap_pyfunction!(decode_synthetic_slice_receipt_json, m)?)?;
    m.add_function(wrap_pyfunction!(build_pool_from_f32_json, m)?)?;
    Ok(())
}

fn parse_shape(shape_json: &str) -> PyResult<ShapeSpec> {
    serde_json::from_str(shape_json).map_err(py_json_err)
}

fn shape_v2(spec: &ShapeSpec) -> PyResult<KvCacheShapeV2> {
    KvCacheShapeV2::new(
        spec.batch,
        spec.layers,
        spec.num_q_heads,
        spec.num_kv_heads,
        spec.seq_len,
        spec.head_dim,
        parse_layout(&spec.layout)?,
        parse_dtype(&spec.dtype)?,
        parse_attention(&spec.attention_kind)?,
    )
    .map_err(py_shape_err)
}

fn legacy_shape_from_json(shape_json: &str) -> PyResult<KvTensorShape> {
    let spec = parse_shape(shape_json)?;
    let _ = shape_v2(&spec)?;
    KvTensorShape::gqa(
        spec.layers,
        spec.num_kv_heads,
        spec.num_kv_heads,
        spec.seq_len,
        spec.head_dim,
        parse_layout(&spec.layout)?,
        parse_dtype(&spec.dtype)?,
    )
    .map_err(py_shape_err)
}

fn parse_layout(value: &str) -> PyResult<KvLayout> {
    match value {
        "layers_heads_tokens_dim" | "LayersHeadsTokensDim" => Ok(KvLayout::LayersHeadsTokensDim),
        "layers_tokens_heads_dim" | "LayersTokensHeadsDim" => Ok(KvLayout::LayersTokensHeadsDim),
        other => Err(PyValueError::new_err(format!(
            "unsupported layout: {other}"
        ))),
    }
}

fn parse_dtype(value: &str) -> PyResult<DType> {
    match value {
        "f32" | "F32" => Ok(DType::F32),
        other => Err(PyValueError::new_err(format!(
            "unsupported dtype for Python sidecar bulk path: {other}"
        ))),
    }
}

fn parse_attention(value: &str) -> PyResult<KvAttentionKind> {
    match value {
        "mha" | "Mha" | "MHA" => Ok(KvAttentionKind::Mha),
        "mqa" | "Mqa" | "MQA" => Ok(KvAttentionKind::Mqa),
        "gqa" | "Gqa" | "GQA" => Ok(KvAttentionKind::Gqa),
        other => Ok(KvAttentionKind::Unsupported(other.to_string())),
    }
}

fn parse_role(value: &str) -> PyResult<KvRole> {
    match value {
        "key" | "Key" => Ok(KvRole::Key),
        "value" | "Value" => Ok(KvRole::Value),
        other => Err(PyValueError::new_err(format!("unsupported role: {other}"))),
    }
}

fn synthetic_blocks(shape: &KvTensorShape) -> PyResult<Vec<ExactKvBlock>> {
    let mut blocks = Vec::new();
    for layer in 0..shape.layers {
        for role in [KvRole::Key, KvRole::Value] {
            let len = shape.layer_element_count(role).map_err(py_shape_err)?;
            let data = (0..len)
                .map(|idx| {
                    let centered = (idx as i32 % 17) - 8;
                    let role_offset = if role == KvRole::Key { 0.0 } else { 0.125 };
                    centered as f32 / 64.0 + role_offset + layer as f32 / 100.0
                })
                .collect::<Vec<_>>();
            blocks.push(
                ExactKvBlock::new(role, LayerId(layer), shape.clone(), data)
                    .map_err(py_poly_err)?,
            );
        }
    }
    Ok(blocks)
}

fn build_pool(shape: KvTensorShape, blocks: Vec<ExactKvBlock>) -> PyResult<SharedKvPool> {
    SharedKvPool::builder()
        .model_fingerprint(ModelFingerprint::new("synthetic:python-sidecar").map_err(py_shape_err)?)
        .tokenizer_fingerprint(
            TokenizerFingerprint::new("synthetic:python-sidecar").map_err(py_shape_err)?,
        )
        .shape(shape)
        .policy(CompressionPolicyV1::alpha_reference())
        .key_codec(Q8KeyCodec::symmetric_per_block())
        .value_codec(RawExactValueCodec)
        .build_from_exact_blocks(blocks)
        .map_err(py_poly_err)
}

fn py_shape_err<E: ToString>(err: E) -> PyErr {
    PyValueError::new_err(err.to_string())
}

fn py_poly_err(err: PolyKvError) -> PyErr {
    PyValueError::new_err(err.to_string())
}

fn py_json_err(err: serde_json::Error) -> PyErr {
    PyValueError::new_err(err.to_string())
}
