use anyhow::anyhow;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use regex::{Captures, Regex};
use tracing::{error, info};

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        // twitter.comまたはx.comのURLを含むメッセージの場合、fxtwitter.comまたはfixupx.comに変換する
        if msg.content.contains("https://twitter.com/") || msg.content.contains("https://x.com/") {
            //複数のURLが含まれている可能性があるので、全て変換する
            let mut result = msg.content.clone();
            for url in msg.content.split_whitespace() {
                if url.contains("https://twitter.com/") || url.contains("https://x.com/") {
                    match convert_twitter_url(url) {
                        Ok(converted_url) => {
                            result = result.replace(url, &converted_url);
                        }
                        Err(e) => {
                            error!("Error: {}", e);
                        }
                    }
                }
            }
            //変換した結果、メッセージが変わっていた場合は、変換後のメッセージを送信する
            if result != msg.content {
                if let Err(why) = msg.channel_id.say(&ctx.http, result).await {
                    error!("Error sending message: {:?}", why);
                }
            }
        }
    }


    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}

// メッセージが "https://twitter.com/username/status/xxxxxx"か "https://x.com/username/status/xxxxxx" の形式を確認し、twitter.comの場合はfxtwitter,comに、x,comの場合はfixupx.comに変換する
fn convert_twitter_url(url: &str) -> anyhow::Result<String> {
    // twitter | x .com
    let re = Regex::new(r"https://(twitter|x)\.com/([a-zA-Z0-9_]+)/status/([0-9]+)")?;
    //twitter.comの場合はfxtwitter,comに、x,comの場合はfixupx.comに変換
    let result = re.replace(url, |caps: &Captures| {
        if caps.get(1).unwrap().as_str() == "twitter" {
            format!("https://fxtwitter.com/{}/status/{}", caps.get(2).unwrap().as_str(), caps.get(3).unwrap().as_str())
        } else {
            format!("https://fixupx.com/{}/status/{}", caps.get(2).unwrap().as_str(), caps.get(3).unwrap().as_str())
        }
    });
    Ok(result.to_string())
}