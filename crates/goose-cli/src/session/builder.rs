use console::style;
use goose::agents::extension::ExtensionError;
use goose::agents::AgentFactory;
use goose::config::{Config, ExtensionManager};
use mcp_client::transport::Error as McpClientError;
use std::path::PathBuf;
use std::process;

use super::output;
use super::storage;
use super::Session;

pub async fn build_session(
    identifier: Option<storage::Identifier>,
    resume: bool,
    extensions: Vec<String>,
    builtins: Vec<String>,
) -> Session {
    // Load config and get provider/model
    let config = Config::global();

    let provider_name: String = config
        .get("GOOSE_PROVIDER")
        .expect("No provider configured. Run 'goose configure' first");

    let model: String = config
        .get("GOOSE_MODEL")
        .expect("No model configured. Run 'goose configure' first");
    let model_config = goose::model::ModelConfig::new(model.clone());
    let provider =
        goose::providers::create(&provider_name, model_config).expect("Failed to create provider");

    // Create the agent
    let agent_version: Option<String> = config.get("GOOSE_AGENT").ok();
    let mut agent = match agent_version {
        Some(version) => AgentFactory::create(&version, provider),
        None => AgentFactory::create(AgentFactory::default_version(), provider),
    }
    .expect("Failed to create agent");

    // Setup extensions for the agent
    for extension in ExtensionManager::get_all().expect("should load extensions") {
        if extension.enabled {
            let config = extension.config.clone();
            agent
                .add_extension(config.clone())
                .await
                .unwrap_or_else(|e| {
                    let err = match e {
                        ExtensionError::Transport(McpClientError::StdioProcessError(inner)) => {
                            inner
                        }
                        _ => e.to_string(),
                    };
                    println!("Failed to start extension: {}, {:?}", config.name(), err);
                    println!(
                        "Please check extension configuration for {}.",
                        config.name()
                    );
                    process::exit(1);
                });
        }
    }

    // Handle session file resolution and resuming
    let session_file = if resume {
        if let Some(identifier) = identifier {
            let session_file = storage::get_path(identifier);
            if !session_file.exists() {
                output::render_error(&format!(
                    "Cannot resume session {} - no such session exists",
                    style(session_file.display()).cyan()
                ));
                process::exit(1);
            }
            session_file
        } else {
            // Try to resume most recent session
            match storage::get_most_recent_session() {
                Ok(file) => file,
                Err(_) => {
                    output::render_error("Cannot resume - no previous sessions found");
                    process::exit(1);
                }
            }
        }
    } else {
        // Create new session with provided name/path or generated name
        let id = match identifier {
            Some(identifier) => identifier,
            None => storage::Identifier::Name(generate_session_name()),
        };
        let session_file = storage::get_path(id);
        create_new_session_file(session_file)
    };

    // Create new session
    let mut session = Session::new(agent, session_file.clone());

    // Add extensions if provided
    for extension_str in extensions {
        if let Err(e) = session.add_extension(extension_str).await {
            eprintln!("Failed to start extension: {}", e);
            process::exit(1);
        }
    }

    // Add builtin extensions
    for builtin in builtins {
        if let Err(e) = session.add_builtin(builtin).await {
            eprintln!("Failed to start builtin extension: {}", e);
            process::exit(1);
        }
    }

    // Add CLI-specific system prompt extension
    session
        .agent
        .extend_system_prompt(super::prompt::get_cli_prompt())
        .await;

    // Only override system prompt if a system override exists
    let system_prompt_file: Option<String> = config.get("GOOSE_SYSTEM_PROMPT_FILE_PATH").ok();
    if let Some(ref path) = system_prompt_file {
        let override_prompt =
            std::fs::read_to_string(path).expect("Failed to read system prompt file");
        session.agent.override_system_prompt(override_prompt).await;
    }

    output::display_session_info(resume, &provider_name, &model, &session_file);
    session
}

fn generate_session_name() -> String {
    use rand::{distributions::Alphanumeric, Rng};
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect()
}

fn create_new_session_file(session_file: PathBuf) -> PathBuf {
    if session_file.exists() {
        eprintln!("Session '{:?}' already exists", session_file);
        process::exit(1);
    }
    session_file
}
