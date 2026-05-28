//! App-level path safety helpers and canonical tool side-effect classification.

use llm_tool_runtime::ToolSideEffectClass;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

pub fn requires_explicit_permit(side_effect: &ToolSideEffectClass) -> bool {
    !matches!(
        side_effect,
        ToolSideEffectClass::ReadOnly | ToolSideEffectClass::Analysis
    )
}

pub fn is_dangerous_without_permit(side_effect: &ToolSideEffectClass) -> bool {
    matches!(
        side_effect,
        ToolSideEffectClass::PreviewWrite | ToolSideEffectClass::Write | ToolSideEffectClass::Admin
    )
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PathSafetyError {
    #[error("path traversal is not allowed")]
    TraversalNotAllowed,
    #[error("path is outside sandbox root {root}")]
    OutsideSandbox { root: String },
    #[error("path is in sensitive prefix {prefix}")]
    SensitivePrefix { prefix: String },
    #[error("path contains hidden or sensitive component {component}")]
    HiddenOrSensitiveComponent { component: String },
}

pub fn path_contains_parent_dir(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

pub fn expand_home_path(path: &str, home: &Path) -> PathBuf {
    if path == "~" {
        return home.to_path_buf();
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return home.join(rest);
    }
    PathBuf::from(path)
}

pub fn validate_sandbox_path(path: &Path, sandbox_root: &Path) -> Result<(), PathSafetyError> {
    if path_contains_parent_dir(path) {
        return Err(PathSafetyError::TraversalNotAllowed);
    }
    if !path.starts_with(sandbox_root) {
        return Err(PathSafetyError::OutsideSandbox {
            root: "<sandbox-root>".into(),
        });
    }

    let relative = path.strip_prefix(sandbox_root).unwrap_or(path);
    for component in relative.components() {
        let Component::Normal(name) = component else {
            continue;
        };
        let name = name.to_string_lossy();
        let folded = name.to_ascii_lowercase();
        if is_sensitive_component(&folded) {
            return Err(PathSafetyError::SensitivePrefix {
                prefix: name.into_owned(),
            });
        }
        if folded.starts_with('.') {
            return Err(PathSafetyError::HiddenOrSensitiveComponent {
                component: name.into_owned(),
            });
        }
    }
    Ok(())
}

fn is_sensitive_component(component: &str) -> bool {
    matches!(
        component,
        ".git"
            | ".git-credentials"
            | ".env"
            | ".env.local"
            | ".npmrc"
            | ".aws"
            | ".ssh"
            | ".gnupg"
            | ".cargo"
            | ".recall"
            | ".password-store"
            | "id_rsa"
            | "id_ed25519"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_effects_require_canonical_permit_classification() {
        assert!(requires_explicit_permit(&ToolSideEffectClass::Write));
        assert!(!requires_explicit_permit(&ToolSideEffectClass::ReadOnly));
        assert!(is_dangerous_without_permit(&ToolSideEffectClass::Admin));
    }

    #[test]
    fn path_safety_rejects_traversal_and_sensitive_prefixes() {
        let home = Path::new("/home/user");

        assert!(path_contains_parent_dir(Path::new("../secret.txt")));
        assert_eq!(expand_home_path("~/repo", home), home.join("repo"));
        assert_eq!(
            validate_sandbox_path(Path::new("/home/user/.ssh/id_rsa"), home),
            Err(PathSafetyError::SensitivePrefix {
                prefix: ".ssh".into()
            })
        );
        assert_eq!(
            validate_sandbox_path(Path::new("/home/user/.npmrc"), home),
            Err(PathSafetyError::SensitivePrefix {
                prefix: ".npmrc".into()
            })
        );
        assert_eq!(
            validate_sandbox_path(Path::new("/home/user/.hidden"), home),
            Err(PathSafetyError::HiddenOrSensitiveComponent {
                component: ".hidden".into()
            })
        );
        assert_eq!(
            validate_sandbox_path(Path::new("/tmp/file.txt"), home),
            Err(PathSafetyError::OutsideSandbox {
                root: "<sandbox-root>".into()
            })
        );
        assert_eq!(
            validate_sandbox_path(Path::new("/home/user/repo"), home),
            Ok(())
        );
    }
}
