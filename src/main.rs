use censor::Censor;
use chrono::{DateTime, Utc};
use poise::serenity_prelude as serenity;
use rand::prelude::*;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::sync::Arc;
use tokio::sync::Mutex;

struct Data {
    swears: Arc<Mutex<HashMap<serenity::UserId, usize>>>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, prefix_command)]
async fn swear_leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    let entries: Vec<(serenity::UserId, usize)> = {
        let data = ctx.data().swears.lock().await;
        let mut vec: Vec<_> = data.iter().map(|(k, v)| (*k, *v)).collect();
        vec.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
        vec
    };

    if entries.is_empty() {
        ctx.say("no one sweared cuz everyone here is a fucking baby")
            .await?;
        return Ok(());
    }

    let mut response = String::from("inmature mfs\n");
    for (i, (user_id, count)) in entries.into_iter().take(10).enumerate() {
        if let Ok(user) = user_id.to_user(ctx.serenity_context()).await {
            response.push_str(&format!("{}. {} — {} swears\n", i + 1, user.name, count));
        }
    }

    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn pi(ctx: Context<'_>) -> Result<(), Error> {
    let mut pi_string = format!("{:.15}", PI); // get Pi to 15 decimal places

    // VERY SMALL CHANCE to mess up a digit
    if rand::rng().random_bool(0.000000001454) {
        let digits: Vec<char> = pi_string.chars().collect();
        let mut rng = rand::rng();

        // pick a random index after the decimal point (skip '3' and '.')
        let idx = rng.random_range(2..digits.len());
        let new_digit = rng.random_range(0..10).to_string().chars().next().unwrap();

        let mut new_pi_string = digits.clone();
        new_pi_string[idx] = new_digit;
        pi_string = new_pi_string.iter().collect();
    }

    ctx.say(format!("pi is: {}", pi_string)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn avatar(
    ctx: Context<'_>,
    #[description = "User mention"] user: Option<serenity::User>,
    #[description = "User ID"] user_id: Option<serenity::UserId>,
) -> Result<(), Error> {
    // Decide whose avatar to show
    let user: serenity::User = if let Some(user) = user {
        user
    } else if let Some(uid) = user_id {
        ctx.serenity_context().http.get_user(uid).await?
    } else {
        ctx.author().clone()
    };

    let avatar_url = user
        .avatar_url()
        .unwrap_or_else(|| user.default_avatar_url());

    // Build the embed
    let embed = serenity::CreateEmbed::new()
        .title(format!("{}'s Avatar", user.name))
        .image(avatar_url);

    // Send it
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());

    // Convert Timestamp -> chrono::DateTime<Utc>
    let datetime: DateTime<Utc> = u.created_at().to_utc();

    let timestamp = datetime.timestamp();

    let response = format!(
        "{}'s account was created at {} (<t:{}:R>)",
        u.name, datetime, timestamp
    );

    ctx.say(response).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![swear_leaderboard, pi, avatar, age],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("-".into()),
                ..Default::default()
            },
            #[allow(unused_variables)]
            event_handler: |ctx, event, _framework, data| {
                Box::pin(async move {
                    if let serenity::FullEvent::Message { new_message } = event {
                        if new_message.author.bot {
                            return Ok(());
                        }

                        let censor = Censor::Standard;
                        if censor.check(&new_message.content) {
                            let mut map = data.swears.lock().await;
                            *map.entry(new_message.author.id).or_insert(0) += 1;
                        }
                    }

                    Ok(())
                })
            },
            ..Default::default()
        })
        .setup(|_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    swears: Arc::new(Mutex::new(HashMap::new())),
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("kablam! {:?}", why);
    }
}
