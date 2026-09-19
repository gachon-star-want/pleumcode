use dotenvy::dotenv;
use futures::StreamExt;
use pleum::agents::{Agent, AgentEvent, ExtensionConfig, SessionConfig};
use pleum::config::{PleumMode, DEFAULT_EXTENSION_DESCRIPTION, DEFAULT_EXTENSION_TIMEOUT};
use pleum::conversation::message::Message;
use pleum::providers::create_with_named_model;
use pleum::session::session_manager::SessionType;
use pleum_providers::databricks::DATABRICKS_DEFAULT_MODEL;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv();

    let provider = create_with_named_model("databricks", Vec::new()).await?;
    let model_config =
        pleum::model_config::model_config_from_user_config("databricks", DATABRICKS_DEFAULT_MODEL)?;

    let agent = Agent::new();

    let session = agent
        .config
        .session_manager
        .create_session(
            PathBuf::default(),
            "max-turn-test".to_string(),
            SessionType::Hidden,
            PleumMode::default(),
        )
        .await?;

    agent
        .update_provider(provider, model_config, &session.id)
        .await?;

    let config = ExtensionConfig::stdio(
        "developer",
        "./target/debug/pleumcode",
        DEFAULT_EXTENSION_DESCRIPTION,
        DEFAULT_EXTENSION_TIMEOUT,
    )
    .with_args(vec!["mcp", "developer"]);
    agent.add_extension(config, &session.id).await?;

    println!("Extensions:");
    for extension in agent.list_extensions().await {
        println!("  {}", extension);
    }

    let session_config = SessionConfig {
        id: session.id,
        schedule_id: None,
        max_turns: None,
        retry_config: None,
    };

    let user_message = Message::user()
        .with_text("can you summarize the readme.md in this dir using just a haiku?");

    let mut stream = agent
        .reply(
            user_message,
            session_config,
            pleum::agents::state_machine::enabled(),
            None,
        )
        .await?;

    while let Some(Ok(AgentEvent::Message(message))) = stream.next().await {
        println!("{}", serde_json::to_string_pretty(&message)?);
        println!("\n");
    }

    Ok(())
}
