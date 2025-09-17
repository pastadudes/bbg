// use censor::Censor;
use chrono::{DateTime, Utc};
use poise::samples::HelpConfiguration;
use poise::serenity_prelude as serenity;
use rand::prelude::*;
use tokio::time::{Duration, Instant, sleep_until};
// use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
use std::f64::consts::PI;
// use std::sync::Arc;
// use tokio::sync::Mutex;
use image::GenericImageView;
use reqwest::blocking::get;
use serenity::model::colour::Color;

struct Data {
    start_time: Instant,
}
// #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
// struct SwearKey {
//     guild_id: u64,
//     user_id: u64,
// }
//
// impl From<(serenity::GuildId, serenity::UserId)> for SwearKey {
//     fn from((g, u): (serenity::GuildId, serenity::UserId)) -> Self {
//         Self {
//             guild_id: g.get(),
//             user_id: u.get(),
//         }
//     }
// }
//
// impl From<&SwearKey> for (serenity::GuildId, serenity::UserId) {
//     fn from(k: &SwearKey) -> Self {
//         (
//             serenity::GuildId::new(k.guild_id),
//             serenity::UserId::new(k.user_id),
//         )
//     }
// }
//
// struct Data {
//     swears: Arc<Mutex<HashMap<SwearKey, usize>>>,
// }
//
// impl Data {
//     /// Save by cloning the map and writing that snapshot to disk.
//     async fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//         // take short lock and clone
//         let snapshot = {
//             let guard = self.swears.lock().await;
//             guard.clone()
//         };
//
//         // ensure parent exists
//         if let Some(parent) = std::path::Path::new(path).parent() {
//             if !parent.as_os_str().is_empty() {
//                 tokio::fs::create_dir_all(parent).await?;
//             }
//         }
//
//         // write to tmp then rename for atomicity
//         let tmp = format!("{}.tmp", path);
//         let json = serde_json::to_string_pretty(&snapshot)?;
//         tokio::fs::write(&tmp, json.as_bytes()).await?;
//         tokio::fs::rename(&tmp, path).await?;
//         Ok(())
//     }
//
//     async fn load(
//         path: &str,
//     ) -> Result<HashMap<SwearKey, usize>, Box<dyn std::error::Error + Send + Sync>> {
//         match tokio::fs::read_to_string(path).await {
//             Ok(content) => Ok(serde_json::from_str(&content)?),
//             Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(HashMap::new()),
//             Err(e) => {
//                 eprintln!("failed to read {}: {:?}", path, e);
//                 Err(Box::new(e))
//             }
//         }
//     }
// }

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// #[poise::command(slash_command, prefix_command)]
// async fn swears(ctx: Context<'_>) -> Result<(), Error> {
//     let entries: Vec<(serenity::UserId, usize)> = {
//         let data = ctx.data().swears.lock().await;
//         let guild_id = ctx.guild_id().unwrap();
//
//         let mut vec: Vec<_> = data
//             .iter()
//             .filter_map(|(k, v)| {
//                 let (g, u): (serenity::GuildId, serenity::UserId) = k.into();
//                 if g == guild_id { Some((u, *v)) } else { None }
//             })
//             .collect();
//
//         vec.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
//         vec
//     };
//
//     if entries.is_empty() {
//         ctx.say("no one sweared cuz everyone here is a baby")
//             .await?;
//         return Ok(());
//     }
//
//     let mut response = String::from("inmature people\n");
//     for (i, (user_id, count)) in entries.into_iter().take(10).enumerate() {
//         if let Ok(user) = user_id.to_user(ctx.serenity_context()).await {
//             response.push_str(&format!("{}. {} — {} swears\n", i + 1, user.name, count));
//         }
//     }
//
//     ctx.say(response).await?;
//     Ok(())
// }

/// 15 decimal pi.  
/// also watch out theres a 0.000000001454% chance of a mutated pi
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

/// shows a user's avatar
#[poise::command(slash_command, prefix_command)]
async fn avatar(
    ctx: Context<'_>,
    #[description = "User mention"] user: Option<serenity::User>,
    #[description = "User ID"] user_id: Option<serenity::UserId>,
) -> Result<(), Error> {
    // decide whose avatar to show
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

    // build the embed
    let embed = serenity::CreateEmbed::new()
        .title(format!("{}'s avatar", user.name))
        .image(avatar_url);

    // send it
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// returns with the age of the discord account
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());

    // convert Timestamp -> chrono::DateTime<Utc>
    let datetime: DateTime<Utc> = u.created_at().to_utc();

    let timestamp = datetime.timestamp();

    let response = format!(
        "{}'s account was created at {} (<t:{}:R>)",
        u.name, datetime, timestamp
    );

    ctx.say(response).await?;

    Ok(())
}

/// NOT FOR PUBLIC USE!!! well nothing is stopping you  
/// anyways it registers guild commands
#[poise::command(slash_command, prefix_command)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// HELP!!! well uh it also tracks edits. you can use -help `command` to get more info about a command
#[poise::command(slash_command, track_edits, prefix_command)]
async fn help(ctx: Context<'_>, command: Option<String>) -> Result<(), Error> {
    let config = HelpConfiguration {
        ephemeral: true,
        include_description: true,
        show_context_menu_commands: true,
        show_subcommands: true,
        __non_exhaustive: (), // Poise devs why tf do we need this??? you even hid it in docs...
        extra_text_at_bottom: "\
            Type -help `command` for more help on a specifc command.
            This command also tracks edits!",
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

/// Returns the uptime of the bot
#[poise::command(slash_command, prefix_command)]
async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    let elapsed = ctx.data().start_time.elapsed();
    let hours = elapsed.as_secs() / 3600;
    let minutes = (elapsed.as_secs() % 3600) / 60;
    let seconds = elapsed.as_secs() % 60;

    ctx.say(format!(
        "uptime: {:02}:{:02}:{:02}",
        hours, minutes, seconds
    ))
    .await?;
    Ok(())
}

/// Sends an embed of the user's info.
#[poise::command(slash_command, prefix_command)]
async fn user(
    ctx: Context<'_>,
    #[description = "User mention"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.unwrap_or_else(|| ctx.author().clone());

    let avatar_url = u.avatar_url().unwrap_or_else(|| u.default_avatar_url());

    // HACK: i mean it works and i think its the only way to get author icon url
    let author = serenity::CreateEmbedAuthor::new(&u.name)
        .icon_url(u.avatar_url().unwrap_or_else(|| u.default_avatar_url()));

    let embed_color =
        tokio::task::block_in_place(|| get_avatar_color(&avatar_url)).unwrap_or(Color::default());
    let embed = serenity::CreateEmbed::new()
        // .author(|a: &mut serenity::CreateEmbedAuthor| a.name(&u.name).icon_url(avatar_url))
        .author(author)
        .color(embed_color)
        .thumbnail(u.avatar_url().unwrap_or_else(|| u.default_avatar_url()))
        .field(
            "user info",
            format!("id: {}\nusername: @{}", u.id, u.name),
            false,
        );

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

// thia function was brought to you by chatgpt
// i mean it works? thankfully no need to read docs lol (bad for me no lie)
fn get_avatar_color(url: &str) -> Result<Color, Box<dyn std::error::Error>> {
    let img_bytes = get(url)?.bytes()?;

    let img = image::load_from_memory(&img_bytes)?;
    let (width, height) = img.dimensions();
    let mut r = 0u64;
    let mut g = 0u64;
    let mut b = 0u64;

    // go through the pixels and calculate average color
    for x in 0..width {
        for y in 0..height {
            let pixel = img.get_pixel(x, y).0; // Get pixel (R, G, B, A)
            r += pixel[0] as u64;
            g += pixel[1] as u64;
            b += pixel[2] as u64;
        }
    }

    // calculate the average
    let num_pixels = (width * height) as u64;
    r /= num_pixels;
    g /= num_pixels;
    b /= num_pixels;

    Ok(Color::from_rgb(r as u8, g as u8, b as u8))
}

/// calls saul
#[poise::command(slash_command, prefix_command)]
async fn call(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Calling Saul...").await?;
    sleep_until(Instant::now() + Duration::from_secs(5)).await;
    ctx.say("Saul: yo").await?;
    Ok(())
}

/// Ping pong! not the game tho it just tells you if the bot is responsive (IN TIME)
#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let instant = Instant::now();
    ctx.say(format!("Pong! {:?}", instant.elapsed())).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                pi(),
                avatar(),
                age(),
                register(),
                help(),
                uptime(),
                user(),
                call(),
                ping(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("-".into()),
                ..Default::default()
            },
            #[allow(unused_variables)]
            event_handler: |ctx, event, framework, data| {
                Box::pin(async move {
                    // if let serenity::FullEvent::Message { new_message } = event {
                    //     if new_message.author.bot || new_message.content.starts_with("-") {
                    //         return Ok(());
                    //     }
                    //
                    //     println!(
                    //         "event_handler triggered for message: {}",
                    //         new_message.content
                    //     );
                    //
                    //     let guild_id = match new_message.guild_id {
                    //         Some(g) => g,
                    //         None => return Ok(()), // skip DMs cuz why would it count in dms?
                    //     };
                    //
                    //     let censor = Censor::Standard + Censor::Sex;
                    //     if censor.check(&new_message.content) {
                    //         let mut map = data.swears.lock().await;
                    //         let key = SwearKey::from((guild_id, new_message.author.id));
                    //         *map.entry(key).or_insert(0) += 1;
                    //
                    //         if let Err(e) = data.save("swears.json").await {
                    //             eprintln!("failed to save swears.json: {:?}", e);
                    //         }
                    //     }
                    // }
                    Ok(())
                })
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                if let Err(e) =
                    poise::builtins::register_globally(ctx, &framework.options().commands).await
                {
                    eprintln!("FAILED to register global commands!!! {:?}", e);
                }
                // let swears = Data::load("swears.json").await?;
                // Ok(Data {
                //     swears: Arc::new(Mutex::new(swears)),
                // })
                Ok(Data {
                    start_time: Instant::now(),
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
