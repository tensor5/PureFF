use anyhow::Context;
use octofer::github::UserId;
use octofer::{Config, Octofer};
use pid1::Pid1Settings;
use std::sync::Arc;
use tracing::info;

mod handlers;
mod messages;
mod octocrab_ext;

use handlers::{
    github_app_authorization_handler, installation_handler, installation_repositories_handler,
    issue_comment_handler, pull_request_handler, push_handler,
};

const NAME: &str = "PureFF";
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct AppData {
    pub name: &'static str,
    pub version: &'static str,
    pub bot_user_id: UserId,
}

fn main() -> anyhow::Result<()> {
    Pid1Settings::new().enable_log(true).launch()?;
    main_inner()
}

#[tokio::main]
async fn main_inner() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env()?;
    config.init_logging();

    let id = std::process::id();

    info!("Starting {NAME} v{VERSION} (PID: {id})");

    let bot_user_id = load_bot_user_id()?;

    let app_data = Arc::new(AppData {
        name: NAME,
        version: VERSION,
        bot_user_id,
    });

    let mut app = Octofer::new(config).await?;

    app.on_github_app_authorization(github_app_authorization_handler, Arc::clone(&app_data))
        .await;
    app.on_installation(installation_handler, Arc::clone(&app_data))
        .await;
    app.on_installation_repositories(installation_repositories_handler, Arc::clone(&app_data))
        .await;
    app.on_issue_comment(issue_comment_handler, Arc::clone(&app_data))
        .await;
    app.on_pull_request(pull_request_handler, Arc::clone(&app_data))
        .await;
    app.on_push(push_handler, app_data).await;

    app.start().await?;

    Ok(())
}

fn load_bot_user_id() -> anyhow::Result<UserId> {
    let bot_user_id_str =
        std::env::var("BOT_USER_ID").context("BOT_USER_ID environment variable not set")?;

    let bot_user_id = bot_user_id_str
        .parse::<u64>()
        .context("BOT_USER_ID must be a valid u64 number")?;

    Ok(UserId(bot_user_id))
}
