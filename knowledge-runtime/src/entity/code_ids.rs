use crate::ids::{EntityId, ScopeKey};
use serde::{Deserialize, Serialize};

/// Discriminant for code-entity types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodeEntityKind {
    Repo,
    Crate,
    Module,
    File,
    Function,
    Type,
    Test,
    Trait,
    Impl,
    Const,
    Macro,
}

impl CodeEntityKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Repo => "repo",
            Self::Crate => "crate",
            Self::Module => "module",
            Self::File => "file",
            Self::Function => "function",
            Self::Type => "type",
            Self::Test => "test",
            Self::Trait => "trait",
            Self::Impl => "impl",
            Self::Const => "const",
            Self::Macro => "macro",
        }
    }
}

/// A code entity with hierarchical path identity.
///
/// The `qualified_path` encodes the full code location, e.g.:
/// `semantic-memory::src/lib.rs::MemoryStore::search`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEntity {
    /// Entity identity (scope-aware).
    pub id: EntityId,
    /// Code entity kind.
    pub kind: CodeEntityKind,
    /// Fully qualified path (crate::module::item or file::item).
    pub qualified_path: String,
    /// Short name (e.g. function name, type name).
    pub short_name: String,
    /// Optional containing file path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// Optional line number in the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

/// Build a canonical `EntityId` for a code entity with scope context.
///
/// Format: `code:{scope_prefix}:{kind}:{qualified_path}`
///
/// The scope prefix uses `repo_id` if available, then falls back to
/// `workspace_id`, then to `namespace`. This ensures the same qualified
/// path in different repos/workspaces produces different IDs.
///
/// This is deterministic — the same inputs always produce the same ID.
pub fn code_entity_id(kind: &CodeEntityKind, qualified_path: &str, scope: &ScopeKey) -> EntityId {
    let scope_prefix = scope
        .repo_id
        .as_deref()
        .or(scope.workspace_id.as_deref())
        .unwrap_or(&scope.namespace);

    EntityId::new(format!(
        "code:{}:{}:{}",
        scope_prefix,
        kind.as_str(),
        qualified_path
    ))
}

/// Extract the display-friendly short path from a code entity ID.
///
/// Returns the `kind:qualified_path` portion without the scope prefix,
/// or `None` if the ID doesn't match the expected format.
pub fn code_entity_display_path(id: &EntityId) -> Option<String> {
    let s = id.as_str();
    let rest = s.strip_prefix("code:")?;
    // Skip scope_prefix (first segment)
    let after_scope = rest.find(':').map(|i| &rest[i + 1..])?;
    Some(after_scope.to_string())
}

/// Parse a code entity ID back into its components.
///
/// Returns `(scope_prefix, kind, qualified_path)` or `None` if the ID
/// does not match the `code:{scope}:{kind}:{path}` format.
pub fn parse_code_entity_id(id: &EntityId) -> Option<(String, CodeEntityKind, String)> {
    let s = id.as_str();
    let rest = s.strip_prefix("code:")?;

    // Split: scope_prefix:kind:path
    let first_colon = rest.find(':')?;
    let scope_prefix = &rest[..first_colon];
    let after_scope = &rest[first_colon + 1..];

    let second_colon = after_scope.find(':')?;
    let kind_str = &after_scope[..second_colon];
    let path = &after_scope[second_colon + 1..];

    let kind = match kind_str {
        "repo" => CodeEntityKind::Repo,
        "crate" => CodeEntityKind::Crate,
        "module" => CodeEntityKind::Module,
        "file" => CodeEntityKind::File,
        "function" => CodeEntityKind::Function,
        "type" => CodeEntityKind::Type,
        "test" => CodeEntityKind::Test,
        "trait" => CodeEntityKind::Trait,
        "impl" => CodeEntityKind::Impl,
        "const" => CodeEntityKind::Const,
        "macro" => CodeEntityKind::Macro,
        _ => return None,
    };

    Some((scope_prefix.to_string(), kind, path.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_id_includes_repo_scope() {
        let scope = ScopeKey {
            namespace: "ns".into(),
            domain: None,
            workspace_id: None,
            repo_id: Some("myrepo".into()),
        };
        let id = code_entity_id(&CodeEntityKind::Function, "lib::search", &scope);
        assert_eq!(id.as_str(), "code:myrepo:function:lib::search");
    }

    #[test]
    fn code_id_falls_back_to_workspace() {
        let scope = ScopeKey {
            namespace: "ns".into(),
            domain: None,
            workspace_id: Some("ws1".into()),
            repo_id: None,
        };
        let id = code_entity_id(&CodeEntityKind::Function, "lib::search", &scope);
        assert_eq!(id.as_str(), "code:ws1:function:lib::search");
    }

    #[test]
    fn code_id_falls_back_to_namespace() {
        let scope = ScopeKey::namespace_only("prod");
        let id = code_entity_id(&CodeEntityKind::Function, "lib::search", &scope);
        assert_eq!(id.as_str(), "code:prod:function:lib::search");
    }

    #[test]
    fn same_path_different_repos_yields_different_ids() {
        let scope_a = ScopeKey {
            namespace: "ns".into(),
            domain: None,
            workspace_id: None,
            repo_id: Some("repo-a".into()),
        };
        let scope_b = ScopeKey {
            namespace: "ns".into(),
            domain: None,
            workspace_id: None,
            repo_id: Some("repo-b".into()),
        };
        let id_a = code_entity_id(&CodeEntityKind::Function, "lib::search", &scope_a);
        let id_b = code_entity_id(&CodeEntityKind::Function, "lib::search", &scope_b);
        assert_ne!(id_a, id_b);
    }

    #[test]
    fn same_scoped_path_remains_stable() {
        let scope = ScopeKey {
            namespace: "ns".into(),
            domain: None,
            workspace_id: None,
            repo_id: Some("repo-a".into()),
        };
        let id1 = code_entity_id(&CodeEntityKind::Function, "lib::search", &scope);
        let id2 = code_entity_id(&CodeEntityKind::Function, "lib::search", &scope);
        assert_eq!(id1, id2);
    }

    #[test]
    fn roundtrip_parse() {
        let scope = ScopeKey {
            namespace: "ns".into(),
            domain: None,
            workspace_id: None,
            repo_id: Some("myrepo".into()),
        };
        let kind = CodeEntityKind::Function;
        let path = "semantic_memory::lib::MemoryStore::search";
        let id = code_entity_id(&kind, path, &scope);

        let (parsed_scope, parsed_kind, parsed_path) = parse_code_entity_id(&id).unwrap();
        assert_eq!(parsed_scope, "myrepo");
        assert_eq!(parsed_kind, kind);
        assert_eq!(parsed_path, path);
    }

    #[test]
    fn display_path_strips_scope() {
        let scope = ScopeKey {
            namespace: "ns".into(),
            domain: None,
            workspace_id: None,
            repo_id: Some("myrepo".into()),
        };
        let id = code_entity_id(&CodeEntityKind::Function, "lib::search", &scope);
        assert_eq!(
            code_entity_display_path(&id).unwrap(),
            "function:lib::search"
        );
    }

    #[test]
    fn parse_invalid_id_returns_none() {
        let bad = EntityId::new("not-a-code-id");
        assert!(parse_code_entity_id(&bad).is_none());
    }

    #[test]
    fn all_kinds_roundtrip() {
        let scope = ScopeKey::namespace_only("ns");
        let kinds = [
            CodeEntityKind::Repo,
            CodeEntityKind::Crate,
            CodeEntityKind::Module,
            CodeEntityKind::File,
            CodeEntityKind::Function,
            CodeEntityKind::Type,
            CodeEntityKind::Test,
            CodeEntityKind::Trait,
            CodeEntityKind::Impl,
            CodeEntityKind::Const,
            CodeEntityKind::Macro,
        ];
        for kind in kinds {
            let id = code_entity_id(&kind, "test::path", &scope);
            let (_, parsed, _) = parse_code_entity_id(&id).unwrap();
            assert_eq!(parsed, kind);
        }
    }
}
