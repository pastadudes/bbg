use censor::Censor;
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
async fn pi(ctx: Context<'_>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![swear_leaderboard(), pi()],

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
