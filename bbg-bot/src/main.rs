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

use bbg_core::{AverageColor, imageops::*};
use chrono::{DateTime, Utc};
use image::{ImageBuffer, ImageFormat, Rgb};
use plotters::prelude::*;
use plotters::style::full_palette::ORANGE;
use poise::samples::HelpConfiguration;
use poise::serenity_prelude as serenity;
use std::io::Cursor;
use tetrio_api::http::parameters::leaderboard_query::LeaderboardType;
use tetrio_api::http::parameters::value_bound_query::*;
use tetrio_api::models::users::user_rank::UserRank;
use tetrio_api::{http::clients::reqwest_client::InMemoryReqwestClient, models::packet::Packet};
use tokio::time::{Duration, Instant, sleep_until};

struct Data {
    start_time: Instant,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// 15 decimal pi.  
/// also watch out theres a 0.000000001454% chance of a mutated pi
#[poise::command(slash_command, prefix_command)]
async fn pi(ctx: Context<'_>) -> Result<(), Error> {
    let pi = bbg_core::pi().await;

    ctx.reply(format!("pi is: {pi}")).await?;
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

    let embed_color = AverageColor::from_image_url(&avatar_url)
        .await?
        .to_embed_color();
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
        extra_text_at_bottom: "\
            Type -help `command` for more help on a specifc command.
            This command also tracks edits!",
        ..Default::default()
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

    let embed_color = AverageColor::from_image_url(&avatar_url)
        .await?
        .to_embed_color();
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
    let result = ImageProcessor::new(url)
        .blur(blur)
        .flip(orientation)
        .grayscale(grayscale)
        .process()
        .await?;

    ctx.send(
        poise::CreateReply::default().attachment(serenity::CreateAttachment::bytes(
            result,
            format!("new!!!_{}", img.filename),
        )),
    )
    .await?;
    Ok(())
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
    let ip = bbg_core::get_random_ipv4().await;

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
                    bbg_core::calculate_tetrio_level(*direct_xp).to_string(),
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

async fn create_job_embed() -> Result<serenity::CreateEmbed, Error> {
    use bbg_core::jobs::JobListings;
    let listings = JobListings::fetch().await?.take(5);
    Ok(listings.to_embed())
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
