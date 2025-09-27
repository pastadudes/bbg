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

use chrono::{DateTime, Utc};
use image::GenericImageView;
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb, imageops};
use plotters::prelude::*;
use poise::samples::HelpConfiguration;
use poise::serenity_prelude as serenity;
use rand::prelude::*;
use reqwest::blocking::get;
use serenity::model::colour::Color;
use std::f64::consts::PI;
use std::io::Cursor;
use tetrio_api::{http::clients::reqwest_client::InMemoryReqwestClient, models::packet::Packet};
use tokio::time::{Duration, Instant, sleep_until};
use tracing::{error, info, trace, warn};

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
    trace!("got pi to 15 numbers");

    // VERY SMALL CHANCE to mess up a digit
    if rand::rng().random_bool(0.000000001454) {
        info!("yo mutated pi just happened");
        let digits: Vec<char> = pi_string.chars().collect();
        let mut rng = rand::rng();

        // pick a random index after the decimal point (skip '3' and '.')
        let idx = rng.random_range(2..digits.len());
        let new_digit = rng.random_range(0..10).to_string().chars().next().unwrap();

        let mut new_pi_string = digits.clone();
        new_pi_string[idx] = new_digit;
        pi_string = new_pi_string.iter().collect();
    }

    if let Err(e) = ctx.reply(format!("pi is: {}", pi_string)).await {
        error!("HEY!!! pi() didn't respond {}", e);
    }
    Ok(())
}

/// shows a user's avatar
#[poise::command(slash_command, prefix_command, aliases("av"))]
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

    let embed_color = tokio::task::block_in_place(|| get_avatar_color(&avatar_url))
        .unwrap_or(serenity::Color::default());
    // build the embed
    let embed = serenity::CreateEmbed::new()
        .title(format!("{}'s avatar", user.name))
        .color(embed_color)
        .image(avatar_url);

    // send it
    if let Err(e) = ctx.send(poise::CreateReply::default().embed(embed)).await {
        error!("HEYY!!! avatar() DIDN'T RESPOND!! {}", e);
    }

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

    if let Err(e) = ctx.reply(response).await {
        error!("HEY!!! age() DIDN'T RESPOND!!!! {}", e);
    }

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
#[poise::command(slash_command, prefix_command, aliases("up", "ut"))]
async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    let elapsed = ctx.data().start_time.elapsed();
    let hours = elapsed.as_secs() / 3600;
    let minutes = (elapsed.as_secs() % 3600) / 60;
    let seconds = elapsed.as_secs() % 60;

    if let Err(e) = ctx
        .reply(format!(
            "uptime: {:02}:{:02}:{:02}",
            hours, minutes, seconds
        ))
        .await
    {
        error!("HEY!!! uptime() DIDN'T RESPOND!! {}", e);
    }
    Ok(())
}

/// Sends an embed of the user's info.
#[poise::command(slash_command, prefix_command, aliases("users", "u"))]
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

    if let Err(e) = ctx.send(poise::CreateReply::default().embed(embed)).await {
        error!("HEYY!! user() DIDNT RESPOND!! {}", e);
    }
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

pub fn process_image(
    url: &str,
    blur: Option<f32>,
    orientation: Option<ImageOrientation>,
    grayscale: Option<bool>,
) -> Result<DynamicImage, Error> {
    let img_bytes = get(url)?.bytes()?;
    let mut img = image::load_from_memory(&img_bytes)?;

    // flip (probably)
    if let Some(orientation) = orientation {
        match orientation {
            ImageOrientation::Horizontal => imageops::flip_horizontal_in_place(&mut img),
            ImageOrientation::Vertical => imageops::flip_vertical_in_place(&mut img),
        }
    }

    // BLUR!!
    if let Some(blur_value) = blur {
        let max_blur = 1000.0;
        let blur_amount = blur_value.min(max_blur);
        img = img.fast_blur(blur_amount);
    }

    #[allow(unused_variables)]
    if let Some(true) = grayscale {
        img = img.grayscale();
    }

    Ok(img)
}

/// Perform operations on an image.
#[poise::command(slash_command, prefix_command)]
async fn imageop(
    ctx: poise::Context<'_, Data, Error>,
    #[description = "The image to process"] img: serenity::Attachment,
    #[description = "The blur amount (optional)"] blur: Option<f32>,
    #[description = "The flip direction (optional)"] orientation: Option<ImageOrientation>, // Horizontal or Vertical
    #[description = "Set to true for grayscale"] grayscale: Option<bool>,
) -> Result<(), Error> {
    // check if the file is an image.
    if !img
        .content_type
        .as_ref()
        // FIX: use is_some_and
        .map_or(false, |ct| ct.starts_with("image/"))
    {
        ctx.say("please provide a valid image file!").await?;
        return Ok(());
    }

    if blur.is_none() && orientation.is_none() && grayscale.is_none() {
        ctx.say("??? bro i aint giving you the same image").await?;
        return Ok(());
    }

    let url = img.url.clone();
    // Here we call our existing image processing function
    let result =
        tokio::task::spawn_blocking(move || process_image(&url, blur, orientation, grayscale))
            .await?;

    match result {
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

#[poise::command(
    prefix_command,
    slash_command,
    subcommands(
        "tetrio_user",
        // "records",
        // "league",
        // "stats",
        "activity",
        // "leaderboard"
    )
)]
async fn tetrio(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

fn calculate_tetrio_level(xp: f64) -> f64 {
    let xp =
        (xp / 500.0).powf(0.6) + (xp / (5000.0 + f64::max(0.0, xp - 4000000.0) / 5000.0)) + 1.0;
    xp.trunc()
}

#[poise::command(prefix_command, slash_command, rename = "user")]
async fn tetrio_user(ctx: Context<'_>, username: String) -> Result<(), Error> {
    let client = InMemoryReqwestClient::default();
    let user = client
        .fetch_user_info(&username)
        .await
        .expect("maybe this crate is a bit too old?");
    match &user {
        Packet {
            data: Some(_data), ..
        } => {
            let embed_author = serenity::CreateEmbedAuthor::new(&_data.username);
            let direct_xp = &_data.xp;
            let mut xp = direct_xp.to_string();
            xp.push_str(" XP");

            let tetrio_info = serenity::CreateEmbed::new()
                .author(embed_author)
                .color(serenity::colours::branding::BLURPLE)
                .field("tetrio user id:", &_data.id, false)
                .field("xp:", xp, false)
                .field(
                    "level:",
                    calculate_tetrio_level(*direct_xp).to_string(),
                    true,
                )
                .field("role", format!("{:?}", &_data.role), false);
            ctx.send(poise::CreateReply::default().embed(tetrio_info))
                .await?;
            Ok(())
        }
        Packet { error, .. } => {
            eprintln!(
                "An error has occured while trying to fetch the user! {:?}",
                error
            );
            Ok(())
        }
    }
}

#[poise::command(slash_command, prefix_command)]
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

            // Create the chart in a blocking task
            let chart_data =
                tokio::task::spawn_blocking(move || create_activity_chart(&activity_f64))
                    .await
                    .map_err(|e| format!("task failed: {}", e))?
                    .map_err(|e| format!("graph creation failed: {}", e))?;

            let attachment = serenity::CreateAttachment::bytes(chart_data, "activity.png");
            let embed = serenity::CreateEmbed::new()
                .title("TETR.IO Server Activity")
                .color(serenity::colours::branding::FUCHSIA)
                .image("attachment://activity.png")
                .description("Server activity over time");

            ctx.send(
                poise::CreateReply::default()
                    .embed(embed)
                    .attachment(attachment),
            )
            .await?;
            Ok(())
        }
        Packet {
            error: Some(err), ..
        } => {
            ctx.say(format!("error fetching activity! {:?}", err))
                .await?;
            Ok(())
        }
        _ => {
            ctx.say("bruh failed to fetch server activity").await?;
            Ok(())
        }
    }
}

type BoxedError = Box<dyn std::error::Error + Send + Sync>; // idk felt sassy and idiomatic today

fn create_activity_chart(data: &[f64]) -> Result<Vec<u8>, BoxedError> {
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
            .caption("TETR.IO Server Activity", ("sans-serif", 25))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(0f64..data.len() as f64, y_min..y_max)?;

        chart
            .configure_mesh()
            .x_desc("Time")
            .y_desc("Players")
            .draw()?;

        chart.draw_series(LineSeries::new(
            data.iter().enumerate().map(|(i, &v)| (i as f64, v)),
            &RED,
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

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    trace!("got discord token");
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
                tetrio_user(),
                activity(),
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
                    warn!("FAILED to register global commands!!! {:?}", e);
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
        error!("kablam! {:?}", why);
    }
}
