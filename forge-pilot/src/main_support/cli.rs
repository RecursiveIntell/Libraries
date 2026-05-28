use forge_pilot::{
    bootstrap_source_workspace, import_recent_forge_bundles, observe_scope,
    render_candidate_report, render_loop_report, render_observation_report, score_targets,
    LoopRunner, PilotHistory,
};
use knowledge_runtime::Scope;
use std::path::{Path, PathBuf};

use super::explain::{
    explain_bootstrap_report, explain_candidates, explain_import_report, explain_loop_report,
    explain_observation,
};
use super::storage::{build_loop_config, normalize_user_path, open_resources};

pub(super) async fn run_command_cli(argv: Vec<String>) -> Result<(), String> {
    let args = CommandArgs::parse(argv)?;
    if args.help {
        print!("{}", CommandArgs::help());
        return Ok(());
    }

    let scope = Scope::new(args.namespace.clone());
    let config = build_loop_config(
        &scope,
        args.workspace_path.clone(),
        &args.memory_dir,
        &args.forge_db,
        args.loop_interval_secs,
        args.max_iterations,
    );
    let resources = open_resources(&args.memory_dir, &args.forge_db, &config)?;

    match args.command.as_str() {
        "observe" => {
            let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
                .await
                .map_err(|error| error.to_string())?;
            if args.json {
                let report = render_observation_report(&observation, true);
                println!("{report}");
            } else {
                println!("{}", explain_observation(&observation));
            }
        }
        "candidates" => {
            let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
                .await
                .map_err(|error| error.to_string())?;
            let candidates = score_targets(&observation, &PilotHistory::default(), &config);
            if args.json {
                let report = render_candidate_report(&candidates, true);
                println!("{report}");
            } else {
                println!("{}", explain_candidates(&candidates));
            }
        }
        "bootstrap-source" => {
            let report = bootstrap_source_workspace(&resources.memory_store, &config)
                .await
                .map_err(|error| error.to_string())?;
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                println!("{}", explain_bootstrap_report(&report));
            }
        }
        "import" => {
            let report = import_recent_forge_bundles(
                &args.namespace,
                &resources.forge_store,
                &resources.memory_store,
                64,
            )
            .await
            .map_err(|error| error.to_string())?;
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                println!("{}", explain_import_report(&report));
            }
        }
        "run" => {
            let mut runner = LoopRunner::new(config, resources);
            let report = runner.run().await.map_err(|error| error.to_string())?;
            if args.json {
                let json_report = render_loop_report(&report, true);
                println!("{json_report}");
            } else {
                println!("{}", explain_loop_report(&report));
            }
        }
        _ => {
            return Err(format!(
                "unknown command `{}`\n\n{}",
                args.command,
                CommandArgs::help()
            ));
        }
    }

    Ok(())
}

pub(super) struct CommandArgs {
    command: String,
    namespace: String,
    memory_dir: PathBuf,
    forge_db: PathBuf,
    workspace_path: String,
    loop_interval_secs: u64,
    max_iterations: u32,
    json: bool,
    help: bool,
}

impl CommandArgs {
    pub(super) fn parse(argv: Vec<String>) -> Result<Self, String> {
        let mut args = Self::default();
        let mut iter = argv.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-h" | "--help" => args.help = true,
                "--json" => args.json = true,
                "--namespace" => {
                    args.namespace = iter
                        .next()
                        .ok_or_else(|| "--namespace requires a value".to_string())?;
                }
                "--memory-dir" => {
                    args.memory_dir = PathBuf::from(
                        iter.next()
                            .ok_or_else(|| "--memory-dir requires a value".to_string())?,
                    );
                }
                "--forge-db" => {
                    args.forge_db = PathBuf::from(
                        iter.next()
                            .ok_or_else(|| "--forge-db requires a value".to_string())?,
                    );
                }
                "--workspace-path" => {
                    args.workspace_path = iter
                        .next()
                        .ok_or_else(|| "--workspace-path requires a value".to_string())?;
                }
                "--workspace" => {
                    args.workspace_path = iter
                        .next()
                        .ok_or_else(|| "--workspace requires a value".to_string())?;
                }
                "--loop-interval" => {
                    args.loop_interval_secs = iter
                        .next()
                        .ok_or_else(|| "--loop-interval requires a value".to_string())?
                        .parse()
                        .map_err(|_| "--loop-interval must be an integer".to_string())?;
                }
                "--max-iterations" => {
                    args.max_iterations = iter
                        .next()
                        .ok_or_else(|| "--max-iterations requires a value".to_string())?
                        .parse()
                        .map_err(|_| "--max-iterations must be an integer".to_string())?;
                }
                value if value.starts_with('-') => {
                    return Err(format!("unknown flag `{value}`\n\n{}", Self::help()));
                }
                value => args.command = value.into(),
            }
        }
        args.memory_dir = normalize_user_path(&args.memory_dir);
        args.forge_db = normalize_user_path(&args.forge_db);
        args.workspace_path = normalize_user_path(Path::new(&args.workspace_path))
            .to_string_lossy()
            .to_string();
        Ok(args)
    }

    pub(super) fn help() -> &'static str {
        "forge-pilot\n\nUSAGE:\n  cargo run -p forge-pilot --                 # interactive plain-English wrapper\n  cargo run -p forge-pilot -- tui             # same interactive wrapper\n  cargo run -p forge-pilot -- <command> [options]\n\nCOMMANDS:\n  observe           Reconstruct and print the current observation\n  candidates        Score and print current target candidates\n  bootstrap-source  Scan the workspace and bootstrap thin imported source state\n  import            Import recent Forge evidence bundles through the canonical export -> bridge -> memory lane\n  run               Execute one bounded pilot loop\n\nOPTIONS:\n  --namespace <name>        Namespace to inspect (default: default)\n  --memory-dir <path>       Semantic memory base dir (default: ./memory)\n  --forge-db <path>         Forge DB path (default: ./forge.db)\n  --workspace-path <path>   Workspace path recorded in reports (default: .)\n  --workspace <path>        Alias for --workspace-path\n  --loop-interval <secs>    Background loop pause for the TUI worker (default: 5)\n  --max-iterations <n>      Max iterations per bounded run (default: 4)\n  --json                    Print raw JSON for command mode\n  -h, --help                Show this help\n"
    }
}

impl Default for CommandArgs {
    fn default() -> Self {
        Self {
            command: "run".into(),
            namespace: "default".into(),
            memory_dir: PathBuf::from("memory"),
            forge_db: PathBuf::from("forge.db"),
            workspace_path: ".".into(),
            loop_interval_secs: 5,
            max_iterations: 4,
            json: false,
            help: false,
        }
    }
}
