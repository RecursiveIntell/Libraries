use super::*;

pub(crate) fn schemas_generate(out: &str) -> Result<String> {
    let root = Path::new(out);
    std::fs::create_dir_all(root)
        .with_context(|| format!("failed to create schema root {}", root.display()))?;
    let documents = generated_schema_documents();
    for document in &documents {
        let path = root.join(&document.registration.schema_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("failed to create schema directory {}", parent.display())
            })?;
        }
        std::fs::write(&path, document.pretty_json())
            .with_context(|| format!("failed to write schema {}", path.display()))?;
    }
    if let Some(document) = documents
        .iter()
        .find(|document| document.registration.family == "artifact-envelope")
    {
        let compatibility_path = root.join("artifact_envelope.schema.json");
        std::fs::write(&compatibility_path, document.pretty_json()).with_context(|| {
            format!(
                "failed to write schema compatibility alias {}",
                compatibility_path.display()
            )
        })?;
    }
    let manifest_path = root.join("generated_schema_manifest_v1.json");
    std::fs::write(&manifest_path, generated_schema_manifest_pretty_json())
        .with_context(|| format!("failed to write manifest {}", manifest_path.display()))?;
    Ok(format!(
        "generated {} schema files into {}",
        documents.len(),
        root.display()
    ))
}

pub(crate) fn schemas_check(root: &str) -> Result<String> {
    let report = schema_check_report(root)?;
    let encoded = serde_json::to_string_pretty(&report)?;
    if !report.compatible {
        bail!("{encoded}");
    }
    Ok(encoded)
}

pub(crate) fn schema_check_report(root: &str) -> Result<SchemaCompatibilityReportV1> {
    let root = Path::new(root);
    let registry = current_artifact_family_registry();
    let documents = generated_schema_documents();
    let mut expected_paths = documents
        .iter()
        .map(|document| document.registration.schema_path.clone())
        .collect::<BTreeSet<_>>();
    expected_paths.insert("artifact_envelope.schema.json".into());
    let path_report = collect_schema_path_report(root)?;
    let actual_paths = path_report.paths;
    let mut checks = Vec::new();
    let mut missing_schema_paths = Vec::new();
    let mut incompatible_schema_paths = Vec::new();

    for document in &documents {
        let relative = document.registration.schema_path.clone();
        let path = root.join(&relative);
        let mut failure_reason_codes = Vec::new();
        let compatible = if !path.exists() {
            missing_schema_paths.push(relative.clone());
            failure_reason_codes.push("registered-schema-missing".into());
            false
        } else {
            let actual = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read schema {}", path.display()))?;
            match parse_strict_json(&actual) {
                Ok(actual_schema) if actual_schema == document.schema => true,
                Ok(_) => {
                    incompatible_schema_paths.push(relative.clone());
                    failure_reason_codes.push("schema-content-drift-without-major-bump".into());
                    false
                }
                Err(_) => {
                    incompatible_schema_paths.push(relative.clone());
                    failure_reason_codes.push("schema-json-invalid-or-duplicate-key".into());
                    false
                }
            }
        };
        for mode in [
            SchemaCompatibilityModeV1::Backward,
            SchemaCompatibilityModeV1::Forward,
            SchemaCompatibilityModeV1::Full,
            SchemaCompatibilityModeV1::Transitive,
        ] {
            checks.push(if compatible {
                SchemaCompatibilityCheckV1::exact(
                    document.registration.family.clone(),
                    document.registration.version,
                    mode,
                )
            } else {
                SchemaCompatibilityCheckV1::incompatible(
                    document.registration.family.clone(),
                    document.registration.version,
                    mode,
                    failure_reason_codes.clone(),
                )
            });
        }
    }
    if let Some(document) = documents
        .iter()
        .find(|document| document.registration.family == "artifact-envelope")
    {
        let relative = "artifact_envelope.schema.json".to_string();
        let path = root.join(&relative);
        if !path.exists() {
            missing_schema_paths.push(relative);
        } else {
            let actual = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read schema {}", path.display()))?;
            match parse_strict_json(&actual) {
                Ok(actual_schema) if actual_schema == document.schema => {}
                Ok(_) | Err(_) => incompatible_schema_paths.push(relative),
            }
        }
    }

    let unregistered_schema_paths = actual_paths
        .difference(&expected_paths)
        .cloned()
        .collect::<Vec<_>>();

    let manifest_path = root.join("generated_schema_manifest_v1.json");
    if !manifest_path.exists() {
        missing_schema_paths.push("generated_schema_manifest_v1.json".into());
    } else {
        let actual = std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("failed to read manifest {}", manifest_path.display()))?;
        match parse_strict_json(&actual) {
            Ok(actual_manifest)
                if actual_manifest
                    == serde_json::to_value(generated_schema_manifest())
                        .unwrap_or(serde_json::Value::Null) => {}
            Ok(_) => incompatible_schema_paths.push("generated_schema_manifest_v1.json".into()),
            Err(_) => incompatible_schema_paths.push("generated_schema_manifest_v1.json".into()),
        }
    }

    Ok(SchemaCompatibilityReportV1::new(
        &registry,
        documents.len(),
        checks,
        missing_schema_paths,
        unregistered_schema_paths,
        incompatible_schema_paths,
        path_report.collision_findings,
    ))
}

pub(crate) struct SchemaPathCollection {
    pub(crate) paths: BTreeSet<String>,
    pub(crate) collision_findings: Vec<SchemaPathCollisionFindingV1>,
}

pub(crate) fn collect_schema_path_report(root: &Path) -> Result<SchemaPathCollection> {
    let mut paths = BTreeSet::new();
    if !root.exists() {
        return Ok(SchemaPathCollection {
            paths,
            collision_findings: Vec::new(),
        });
    }
    collect_schema_paths_inner(root, root, &mut paths)?;
    let mut folded = BTreeMap::<String, Vec<String>>::new();
    for path in &paths {
        folded
            .entry(path.to_ascii_lowercase())
            .or_default()
            .push(path.clone());
    }
    let collision_findings = folded
        .into_iter()
        .filter_map(|(normalized, paths)| {
            (paths.len() > 1).then(|| SchemaPathCollisionFindingV1::new(normalized, paths))
        })
        .collect();
    Ok(SchemaPathCollection {
        paths,
        collision_findings,
    })
}

pub(crate) fn collect_schema_paths_inner(
    root: &Path,
    current: &Path,
    paths: &mut BTreeSet<String>,
) -> Result<()> {
    for entry in std::fs::read_dir(current)
        .with_context(|| format!("failed to read schema directory {}", current.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_schema_paths_inner(root, &path, paths)?;
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".schema.json"))
        {
            paths.insert(relative_path(root, &path)?);
        }
    }
    Ok(())
}

pub(crate) fn relative_path(root: &Path, path: &Path) -> Result<String> {
    Ok(path
        .strip_prefix(root)
        .with_context(|| format!("failed to relativize {}", path.display()))?
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}
