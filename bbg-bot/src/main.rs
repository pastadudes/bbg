// bbg, a discord bot (that does basic things)
// Copyright (C) 2025 pastaya
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use bbg_core::{calculate_tetrio_level, get_avatar_color};
use chrono::{DateTime, Utc};
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb, imageops};
use plotters::prelude::*;
use plotters::style::full_palette::ORANGE;
use poise::samples::HelpConfiguration;
use poise::serenity_prelude as serenity;
use rand::prelude::*;
use serde::Deserialize;
// use serde_json::{Deserializer, Serializer};
use serenity::model::colour::Color;
use std::f64::consts::PI;
use std::io::Cursor;
use tetrio_api::http::parameters::leaderboard_query::LeaderboardType;
use tetrio_api::http::parameters::value_bound_query::*;
use tetrio_api::models::users::user_rank::UserRank;
use tetrio_api::{http::clients::reqwest_client::InMemoryReqwestClient, models::packet::Packet};
use tokio::time::{Duration, Instant, sleep_until};

struct Data {
    start_time: Instant,
}

/// for use in flip_image()  
/// basically technically practically defines 2 parameters
#[derive(Debug, poise::ChoiceParameter)]
pub enum ImageOrientation {
    #[name = "horizontally"]
    Horizontal,
    #[name = "vertically"]
    Vertical,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

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

    ctx.reply(format!("pi is: {}", pi_string)).await?;
    Ok(())
}

/// shows a user's avatar
#[poise::command(slash_command, prefix_command, aliases("av"))]
async fn avatar(
    ctx: Context<'_>,
    #[description = "user mention"] user: Option<serenity::User>,
    #[description = "user ID"] user_id: Option<serenity::UserId>,
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

    let embed_color = get_avatar_color(&avatar_url)
        .await
        .unwrap_or(serenity::Color::default());
    // build the embed
    let embed = serenity::CreateEmbed::new()
        .title(format!("{}'s avatar", user.name))
        .color(embed_color)
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

    ctx.reply(response).await?;

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
        ephemeral: false,
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
#[poise::command(slash_command, prefix_command, aliases("up", "ut"))]
async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    let elapsed = ctx.data().start_time.elapsed();
    let hours = elapsed.as_secs() / 3600;
    let minutes = (elapsed.as_secs() % 3600) / 60;
    let seconds = elapsed.as_secs() % 60;

    ctx.reply(format!(
        "uptime: {:02}:{:02}:{:02}",
        hours, minutes, seconds
    ))
    .await?;
    Ok(())
}

/// Sends an embed of the user's info.
#[poise::command(slash_command, prefix_command, aliases("users", "u"))]
async fn user(
    ctx: Context<'_>,
    #[description = "User mention"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user = user.unwrap_or_else(|| ctx.author().clone());

    let avatar_url = user
        .avatar_url()
        .unwrap_or_else(|| user.default_avatar_url());

    // HACK: i mean it works and i think its the only way to get author icon url
    let author = serenity::CreateEmbedAuthor::new(&user.name).icon_url(
        user.avatar_url()
            .unwrap_or_else(|| user.default_avatar_url()),
    );

    let embed_color = get_avatar_color(&avatar_url)
        .await
        .unwrap_or(Color::default());
    let embed = serenity::CreateEmbed::new()
        // .author(|a: &mut serenity::CreateEmbedAuthor| a.name(&user.name).icon_url(avatar_url))
        .author(author)
        .color(embed_color)
        .thumbnail(
            user.avatar_url()
                .unwrap_or_else(|| user.default_avatar_url()),
        )
        .field(
            "user info",
            format!("id: {}\nusername: @{}", user.id, user.name),
            false,
        );

    ctx.send(poise::CreateReply::default().embed(embed).reply(true))
        .await?;

    Ok(())
}

/// calls saul
#[poise::command(slash_command, prefix_command)]
async fn call(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("Calling Saul...").await?;
    sleep_until(Instant::now() + Duration::from_secs(5)).await;
    ctx.say("Saul: yo").await?;
    Ok(())
}

/// Ping pong! not the game tho it just tells you if the bot is responsive (IN TIME)
#[poise::command(prefix_command, slash_command, aliases("p"))]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    // TODO: uhh add error handling
    let before_timestamp = ctx.created_at();

    // send an initial reply and get a handle to it
    let reply_handle = ctx.reply("Pong!").await?;

    // retrieve the full message object from the handle
    let message = reply_handle.message().await?;

    // get the timestamp of the bot's reply message
    let after_timestamp = message.timestamp;

    // convert both timestamps into chrono to subtract
    let before: chrono::DateTime<chrono::Utc> = before_timestamp.to_utc();
    let after: chrono::DateTime<chrono::Utc> = after_timestamp.to_utc();

    let latency = after - before;

    let response_content = format!("Pong! Latency: `{}ms`", latency.num_milliseconds());

    let builder = poise::CreateReply::default().content(response_content);

    reply_handle.edit(ctx, builder).await?;

    Ok(())
}

async fn process_image(
    url: &str,
    blur: Option<f32>,
    orientation: Option<ImageOrientation>,
    grayscale: Option<bool>,
) -> Result<DynamicImage, Error> {
    let image_bytes = reqwest::get(url).await?.bytes().await?;
    let mut loaded_image = image::load_from_memory(&image_bytes)?;

    // flip (probably)
    if let Some(orientation) = orientation {
        match orientation {
            ImageOrientation::Horizontal => imageops::flip_horizontal_in_place(&mut loaded_image),
            ImageOrientation::Vertical => imageops::flip_vertical_in_place(&mut loaded_image),
        }
    }

    // BLUR!!
    if let Some(blur_value) = blur {
        let max_blur = 1000.0;
        let blur_amount = blur_value.min(max_blur);
        loaded_image = loaded_image.fast_blur(blur_amount);
    }

    #[allow(unused_variables)]
    if let Some(true) = grayscale {
        loaded_image = loaded_image.grayscale();
    }

    Ok(loaded_image)
}

/// Perform operations on an image.
#[poise::command(slash_command, prefix_command)]
async fn imageop(
    ctx: Context<'_>,
    #[description = "the image to process"] img: serenity::Attachment,
    #[description = "the blur amount (optional)"] blur: Option<f32>,
    #[description = "the flip direction (optional)"] orientation: Option<ImageOrientation>, // Horizontal or Vertical
    #[description = "set to true for grayscale"] grayscale: Option<bool>,
) -> Result<(), Error> {
    // check if the file is an image.
    if !img
        .content_type
        .as_ref()
        // FIX: use is_some_and
        .is_some_and(|ct| ct.starts_with("image/"))
    {
        ctx.say("please provide a valid image file!").await?;
        return Ok(());
    }

    if blur.is_none() && orientation.is_none() && grayscale.is_none() {
        ctx.say("??? bro i aint giving you the same image").await?;
        return Ok(());
    }

    let url = img.url;
    // Here we call our existing image processing function
    let result = process_image(&url, blur, orientation, grayscale);

    match result.await {
        Ok(finished_img) => {
            // prepare for lots of uhhhh something
            let mut img_data: Vec<u8> = Vec::new();
            finished_img.write_to(&mut Cursor::new(&mut img_data), image::ImageFormat::Png)?;
            let reply = poise::CreateReply::default().attachment(
                serenity::CreateAttachment::bytes(img_data, format!("new!!!_{}", img.filename)),
            );
            ctx.send(reply).await?;
            Ok(())
        }
        Err(e) => {
            ctx.say(format!("kaBOOMM! {}", e)).await?;
            Err(e)
        }
    }
}

/// required by the agpl  
/// if a fork of my bot doesn't include this pls contact me  
/// @pastaya
#[poise::command(prefix_command, slash_command)]
async fn source(ctx: Context<'_>) -> Result<(), Error> {
    // ehh this doesn't need error handling
    ctx.reply("https://github.com/pastadudes/bbg").await?;
    Ok(())
}

/// Get a random ip address
#[poise::command(prefix_command, slash_command)]
async fn ipv4(ctx: Context<'_>) -> Result<(), Error> {
    let mut rng = rand::rngs::StdRng::from_os_rng();

    let octet1: u8 = rng.random_range(0..=255);
    let octet2: u8 = rng.random_range(0..=255);
    let octet3: u8 = rng.random_range(0..=255);
    let octet4: u8 = rng.random_range(0..=255);

    let ip = format!("{}.{}.{}.{}", octet1, octet2, octet3, octet4);

    ctx.reply(format!(
        "heres a vaild ip address (may not be online): {}",
        ip
    ))
    .await?;
    Ok(())
}

/// tetrio related commands, DO NOT USE STANDALONE!! YOU MUST SPECIFY A SUBCOMMAND!!
#[poise::command(
    prefix_command,
    slash_command,
    subcommands(
        "tetrio_user",
        // "records",
        // "league",
        // "stats",
        "activity",
        "leaderboard"
    )
)]
async fn tetrio(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("you forgot the subcommand...").await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, rename = "user")]
async fn tetrio_user(ctx: Context<'_>, username: String) -> Result<(), Error> {
    let client = InMemoryReqwestClient::default();
    let user = client.fetch_user_info(&username).await?;
    match &user {
        Packet {
            data: Some(data), ..
        } => {
            let embed_author = serenity::CreateEmbedAuthor::new(&data.username);
            let direct_xp = &data.xp;
            let mut xp = direct_xp.to_string();
            xp.push_str(" XP");

            let tetrio_info = serenity::CreateEmbed::new()
                .author(embed_author)
                .color(serenity::colours::branding::BLURPLE)
                .field("tetrio user id:", &data.id, false)
                .field("xp:", xp, false)
                .field(
                    "level:",
                    calculate_tetrio_level(*direct_xp).to_string(),
                    true,
                )
                .field("role", format!("{:?}", &data.role), false);
            ctx.send(poise::CreateReply::default().embed(tetrio_info))
                .await?;
            Ok(())
        }
        Packet { error, .. } => {
            eprintln!(
                "an error has occured while trying to fetch the user! {:?}",
                error
            );
            Ok(())
        }
    }
}

/// Shows general activity of tetrio
#[poise::command(slash_command, prefix_command, broadcast_typing)]
async fn activity(ctx: Context<'_>) -> Result<(), Error> {
    let client = InMemoryReqwestClient::default();

    let server_activity = client
        .fetch_general_activity()
        .await
        .map_err(|e| format!("failed to fetch activity ;( {}", e))?;

    match server_activity {
        Packet {
            data: Some(data), ..
        } => {
            let activity_f64: Vec<f64> = data.activity.iter().map(|&x| x as f64).collect();

            // create the chart in a blocking task
            let chart_data =
                tokio::task::spawn_blocking(move || create_activity_chart(&activity_f64))
                    .await
                    .map_err(|e| format!("task failed: {}", e))?
                    .map_err(|e| format!("graph creation failed: {}", e))?;

            let attachment = serenity::CreateAttachment::bytes(chart_data, "activity.png");
            ctx.send(poise::CreateReply::default().attachment(attachment))
                .await?;
            Ok(())
        }
        Packet { error, .. } => {
            ctx.say(format!("error fetching activity! {:?}", error))
                .await?;
            Ok(())
        }
    }
}

fn create_activity_chart(data: &[f64]) -> Result<Vec<u8>, Error> {
    const W: u32 = 800;
    const H: u32 = 400;
    const BYTES_PER_PIXEL: usize = 3;

    // 1. raw RGB buffer (plotters wants &mut [u8])
    let mut raw = vec![0u8; (W * H) as usize * BYTES_PER_PIXEL];

    {
        // 2. draw into that raw buffer (probably)
        let root = BitMapBackend::with_buffer(&mut raw, (W, H)).into_drawing_area();
        root.fill(&WHITE)?;

        let (min_val, max_val) = match (
            data.iter().cloned().reduce(f64::min),
            data.iter().cloned().reduce(f64::max),
        ) {
            (Some(min), Some(max)) => (min, max),
            _ => return Err("no data".into()),
        };
        let pad = (max_val - min_val) * 0.1;
        let y_min = (min_val - pad).max(0.0);
        let y_max = max_val + pad;

        let mut chart = ChartBuilder::on(&root)
            .caption("tetrio server activity", ("sans-serif", 25))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(0f64..data.len() as f64, y_min..y_max)?;

        chart
            .configure_mesh()
            .x_desc("time")
            .y_desc("players")
            .draw()?;

        chart.draw_series(LineSeries::new(
            data.iter().enumerate().map(|(i, &v)| (i as f64, v)),
            &ORANGE,
        ))?;

        root.present()?;
    }

    // 3. wrap raw rgb bytes in an `RgbImage` and encode to png
    let rgb_img: ImageBuffer<Rgb<u8>, _> =
        ImageBuffer::from_raw(W, H, raw).ok_or("buffer size mismatch")?;
    let mut png_bytes = Vec::new();
    rgb_img.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)?;

    Ok(png_bytes)
}

/// returns an embed of the top 20 players in tetra league
#[poise::command(prefix_command, slash_command)]
async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    let client = &InMemoryReqwestClient::default();
    let tetrio_leaderboard = client
        .fetch_leaderboard(
            LeaderboardType::League,
            ValueBoundQuery::NotBound {
                limit: None,
                country: None,
            },
            None,
        )
        .await?;

    match tetrio_leaderboard {
        Packet {
            data: Some(data), ..
        } => {
            // Build an embed with the top N entries
            let mut embed = serenity::CreateEmbed::default()
                .title("tetrio leaderboard")
                .color(serenity::colours::branding::GREEN);

            for (i, entry) in data.entries.iter().enumerate().take(20) {
                embed = embed.field(
                    format!("#{} {}", i + 1, entry.username),
                    format!(
                        "tr: {:.2} | rank: {} | country: {}",
                        entry.league.tr,
                        rank_label(entry.league.rank.as_ref()),
                        entry.country.clone().unwrap_or_else(|| "??".into())
                    ),
                    false,
                );
            }

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
        Packet { error, .. } => {
            ctx.say(format!("error fetching leaderboard! {:?}", error))
                .await?;
            return Ok(());
        }
    }
}

fn rank_label(rank: Option<&UserRank>) -> &'static str {
    match rank {
        Some(UserRank::XPlus) => "X+",
        Some(UserRank::X) => "X",
        Some(UserRank::U) => "U",
        Some(UserRank::SS) => "SS",
        Some(UserRank::SPlus) => "S+",
        Some(UserRank::S) => "S",
        Some(UserRank::SMinus) => "S-",
        Some(UserRank::APlus) => "A+",
        Some(UserRank::A) => "A",
        Some(UserRank::AMinus) => "A-",
        Some(UserRank::BPlus) => "B+",
        Some(UserRank::B) => "B",
        Some(UserRank::BMinus) => "B-",
        Some(UserRank::CPlus) => "C+",
        Some(UserRank::C) => "C",
        Some(UserRank::CMinus) => "C-",
        Some(UserRank::DPlus) => "D+",
        Some(UserRank::D) => "D",
        Some(UserRank::Z) => "Unranked",
        Some(UserRank::Unknown(_)) => "???",
        None => "???",
    }
}

#[derive(Deserialize, Debug)]
struct Jobs {
    company_name: String,
    title: String,
    description: String,
    remote: bool,
    tags: Vec<String>,
    url: String,
    job_types: Vec<String>,
    location: String,
    created_at: u64,
}

// needed cuz there are multiple fields in the api response
#[derive(Deserialize, Debug)]
struct JobListings {
    data: Vec<Jobs>,
}

async fn create_job_embed() -> Result<serenity::CreateEmbed, Error> {
    use nanohtml2text::html2text;
    use reqwest::ClientBuilder;

    let client = ClientBuilder::new()
        .user_agent("contact@pastaya.net if im being too spammy")
        .build()?;

    let response = client
        .get("https://arbeitnow.com/api/job-board-api")
        .send()
        .await?;
    let response = response.error_for_status()?;
    let data: JobListings = response.json().await?;

    let mut description = String::new();

    for (i, job) in data.data.iter().take(5).enumerate() {
        let description_text = html2text(&job.description);
        let truncated_desc = if description_text.chars().count() > 150 {
            format!(
                "{}...",
                &description_text.chars().take(150).collect::<String>()
            )
        } else {
            description_text
        };

        description.push_str(&format!(
            "**{}. {}**\n\
             **company**: {}\n\
             **location**: {}\n\
             **description**: {}\n\
             **remote**: {}\n\
             **tags**: {}\n\
             **job types**: {}\n\
             **posted** <t:{}:R>\n\
             [view job]({})\n\n",
            i + 1,
            job.title,
            job.company_name,
            job.location,
            truncated_desc,
            if job.remote { "yes" } else { "no" },
            job.tags.join(", "),
            job.job_types.join(", "),
            job.created_at,
            job.url
        ));
    }

    let embed = serenity::CreateEmbed::default()
        .title("latest jobs (top 5)")
        .description(description)
        .color(serenity::colours::branding::WHITE);

    Ok(embed)
}

/// shows 5 job listings from arbeitnow.com
#[poise::command(slash_command, prefix_command, aliases("j*bs"))]
async fn jobs(ctx: Context<'_>) -> Result<(), Error> {
    let embed = create_job_embed().await?;
    let builder = poise::CreateReply::default().embed(embed).reply(true);
    ctx.send(builder).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
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
                source(),
                imageop(),
                ipv4(),
                tetrio(),
                jobs(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("-".into()),
                ..Default::default()
            },
            event_handler: |_ctx, _event, _framework, _data| {
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
        .setup(|_ctx, _ready, _framework| {
            Box::pin(async move {
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
        .expect("error creating client");

    if let Err(why) = client.start().await {
        eprintln!("kablam! {:?}", why);
    }
}
