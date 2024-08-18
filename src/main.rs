use rcon;
use tokio;
use poise::serenity_prelude as serenity;

struct Data {
    rconn_addr: String,
    rconn_password: String,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let rconn_addr = "127.0.0.1:25575".to_owned();
    let rconn_password = std::env::var("MINECRAFT_RCONN_PASSWORD").expect("Please set MINECRAFT_RCONN_PASSWORD env var");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![whitelist(), help()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data{
                    rconn_addr,
                    rconn_password,
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn whitelist(
    ctx: Context<'_>,
    #[description = "Player name to whitelist"]
    player: String,
) -> Result<(), Error> {

    if player.is_empty() {
        ctx.say("Player name is required").await?;
        return Ok(());
    }
    // validate
    if !player.chars().all(|x| x.is_alphanumeric() || x == '_') {
        ctx.say("Invalid player name, should match [A-Z0-9_]+").await?;
        return Ok(());
    }

    let mut conn =  <rcon::Connection<tokio::net::TcpStream>>::builder()
        .enable_minecraft_quirks(true)
        .connect(&ctx.data().rconn_addr, &ctx.data().rconn_password).await?;

    let resp = conn.cmd(&format!("whitelist add {player}")).await?;

    ctx.say(format!("{resp}")).await?;
    Ok(())
}
