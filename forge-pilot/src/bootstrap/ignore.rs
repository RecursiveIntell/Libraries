use std::fs;
use std::path::Path;

pub(crate) fn should_skip_workspace_dir(path: &Path, excluded_memory_dir: Option<&Path>) -> bool {
    if excluded_memory_dir
        .is_some_and(|memory_dir| fs::canonicalize(path).ok().as_deref() == Some(memory_dir))
    {
        return true;
    }

    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                ".git"
                    | ".forge-pilot"
                    | ".next"
                    | ".parser-lib"
                    | ".turbo"
                    | "build"
                    | "coverage"
                    | "dist"
                    | "memory"
                    | "node_modules"
                    | "target"
            )
        })
}
