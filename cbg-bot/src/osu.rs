use crate::{Context, Error};
use cbg_core::AverageColor;
use cbg_core::osu::*;
use poise::{command, serenity_prelude as serenity};

#[command(
    prefix_command,
    slash_command,
    subcommands("user", "score", "beatmap", "rank")
)]
pub async fn osu(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("you forgot the subcommand...").await?;
    Ok(())
}

#[command(prefix_command, slash_command, broadcast_typing)]
async fn user(
    ctx: Context<'_>,
    #[description = "user identifier (ID or username)"] identifier: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let user_identifier = identifier
        .parse::<u32>()
        .map_or_else(|_| UserIdentifier::Username(identifier), UserIdentifier::Id);

    let osu = OsuClient::from_env().await?;
    let user = osu.get_user(user_identifier).await?;

    // calculating the amount of digits (we can't use len since "???" would be 3 digits and if the digit is only 1 it would be like 1 digits which is weird)
    let (global_rank_str, rank_digits) = match user.global_rank {
        Some(rank) => {
            let rank_str = rank.to_string();
            let digits = match rank {
                1 => "1 digit".to_string(),
                _ => format!("{} digits", rank_str.len()),
            };
            (rank_str, digits)
        }
        None => ("???".to_string(), "unknown rank...".to_string()),
    };

    let embed = serenity::CreateEmbed::default()
        .author(serenity::CreateEmbedAuthor::new(&user.username).icon_url(&user.avatar_url))
        .color(
            AverageColor::from_image_url(&user.avatar_url)
                .await?
                .to_embed_color(),
        )
        .field("user:", user.username, false)
        .field("country code:", user.country_code, false)
        .field("pp:", format!("{} pp", user.pp), false)
        .field(
            "global rank:",
            format!("#{global_rank_str} ({})", rank_digits),
            false,
        )
        .field("play count:", user.play_count.to_string(), false)
        .field("accuracy:", format!("{}%", user.accuracy), false);

    ctx.send(poise::CreateReply::default().embed(embed).reply(true))
        .await?;

    Ok(())
}

// BAZINGA! almost 150 lines function
#[command(prefix_command, slash_command, broadcast_typing)]
async fn score(
    ctx: Context<'_>,
    #[description = "user identifier (ID or username)"] identifier: String,
    #[description = "score type (best, recent, firsts)"] score_type: Option<ScoreType>,
    #[description = "number of scores (1-5)"] count: Option<usize>,
) -> Result<(), Error> {
    ctx.defer().await?;
    let user_identifier = identifier.parse::<u32>().map_or_else(
        |_| UserIdentifier::Username(identifier.clone()),
        UserIdentifier::Id,
    );

    let osu = OsuClient::from_env().await?;
    let score_type = score_type.unwrap_or(ScoreType::Best);

    let count = count.unwrap_or(3).min(5);
    let scores = osu
        .get_user_scores(user_identifier, score_type, Some(count))
        .await?;

    if scores.is_empty() {
        ctx.say("no scores found for this user!").await?;
        return Ok(());
    }

    let user = &scores[0].user;

    // send initial embeds without beatmap info
    let mut initial_embeds = Vec::new();

    for (i, score) in scores.iter().enumerate() {
        let beatmap_info = if let Some(beatmap_id) = score.beatmap_id {
            format!("beatmap id: {} (fetching details...)", beatmap_id)
        } else {
            "unknown beatmap!".to_string()
        };

        let embed = serenity::CreateEmbed::default()
            .title(format!("score #{}", i + 1))
            .description(beatmap_info)
            .field("player", &user.username, false)
            .field("score", format!("{:?}", score.score), false)
            .field(
                "pp",
                score
                    .pp
                    .map_or("???".to_string(), |pp| format!("{:.2}", pp)),
                false,
            )
            .field("accuracy", format!("{:.2}%", score.accuracy), false)
            .field(
                "combo",
                format!(
                    "{}x{}",
                    score.max_combo,
                    if score.perfect { " 🔥" } else { "" }
                ),
                false,
            )
            .field("rank", &score.rank, false)
            .field("mods", &score.mods, false)
            .color(match score.rank.as_str() {
                "X" | "XH" => serenity::Color::from_rgb(255, 215, 0),
                "S" | "SH" => serenity::Color::LIGHT_GREY,
                "A" => serenity::Color::KERBAL,
                "B" => serenity::Color::from_rgb(255, 223, 0),
                "C" => serenity::Color::ORANGE,
                "D" => serenity::Color::RED,
                _ => serenity::Color::DARK_GREY,
            });

        initial_embeds.push(embed);
    }

    // send initial message
    let reply = ctx
        .send(poise::CreateReply {
            embeds: initial_embeds,
            reply: true,
            ..Default::default()
        })
        .await?;
    let mut message = reply.message().await?;

    // update with beatmap info
    let mut updated_embeds = Vec::new();

    for (index, score) in scores.iter().enumerate() {
        let beatmap_info = if let Some(beatmap_id) = score.beatmap_id {
            match osu.get_beatmap(beatmap_id as u32).await {
                Ok(beatmap) => {
                    format!(
                        "{} - {} [{}]",
                        beatmap.artist, beatmap.title, beatmap.version
                    )
                }
                Err(_) => {
                    format!("beatmap id: {} (failed to fetch)", beatmap_id)
                }
            }
        } else {
            "unknown beatmap!".to_string()
        };

        let embed = serenity::CreateEmbed::default()
            .title(format!("score #{}", index + 1))
            .description(beatmap_info)
            .field("player", &user.username, false)
            .field("score", format!("{:?}", score.score), false)
            .field(
                "pp",
                score
                    .pp
                    .map_or("???".to_string(), |pp| format!("{:.2}", pp)),
                false,
            )
            .field("accuracy", format!("{:.2}%", score.accuracy), false)
            .field(
                "combo",
                format!(
                    "{}x{}",
                    score.max_combo,
                    if score.perfect { " 🔥" } else { "" }
                ),
                false,
            )
            .field("rank", &score.rank, false)
            .field("mods", &score.mods, false)
            .color(match score.rank.as_str() {
                "X" | "XH" => serenity::Color::from_rgb(255, 215, 0),
                "S" | "SH" => serenity::Color::LIGHT_GREY,
                "A" => serenity::Color::KERBAL,
                "B" => serenity::Color::from_rgb(255, 223, 0),
                "C" => serenity::Color::ORANGE,
                "D" => serenity::Color::RED,
                _ => serenity::Color::DARK_GREY,
            });

        updated_embeds.push(embed);
    }

    // edit the message with updated embeds
    message
        .to_mut()
        .edit(ctx, serenity::EditMessage::default().embeds(updated_embeds))
        .await?;

    Ok(())
}

/// osu: fetches beatmap info by id
#[command(prefix_command, slash_command, broadcast_typing)]
async fn beatmap(
    ctx: Context<'_>,
    #[description = "beatmap ID"] beatmap_id: u32,
) -> Result<(), Error> {
    ctx.defer().await?;
    let osu = OsuClient::from_env().await?;
    let beatmap = osu.get_beatmap(beatmap_id).await?;

    let embed = serenity::CreateEmbed::default()
        .title(&beatmap.title)
        .field("id:", beatmap.id.to_string(), false)
        .field("artist:", beatmap.artist, false)
        .field("mapper:", beatmap.creator, false)
        .field("version:", beatmap.version, false)
        .field("stars", format!("{} ⭐", beatmap.stars), false)
        .field("bpm:", beatmap.bpm.to_string(), false)
        .field("ar:", beatmap.ar.to_string(), false)
        .field("cs:", beatmap.cs.to_string(), false)
        .field("hp drain:", beatmap.hp.to_string(), false)
        .field("od:", beatmap.od.to_string(), false);

    let mut reply = poise::CreateReply::default().embed(embed).reply(true);
    // Add the image only if it's available
    if let Some(background_image) = beatmap.background_image {
        let attachment =
            serenity::CreateAttachment::bytes(background_image, format!("{}.jpeg", &beatmap.title));
        // TODO: include hte image in the embed
        reply = reply.attachment(attachment);
        // // TODO: add average color when AverageColor implements bytes
        // embed = embed.color()
    }
    ctx.send(reply).await?;
    Ok(())
}

/// osu: get rank of any user (country rank too)
#[command(prefix_command, slash_command)]
async fn rank(
    ctx: Context<'_>,
    #[description = "user identifier (ID or username)"] identifier: String,
    #[description = "optional: country code for country rank"] country: Option<String>,
) -> Result<(), Error> {
    let user_identifier = identifier
        .parse::<u32>()
        .map_or_else(|_| UserIdentifier::Username(identifier), UserIdentifier::Id);

    let osu = OsuClient::from_env().await?;
    let user = osu.get_user(user_identifier).await?;

    if let Some(country_code) = country {
        // fetch user country rank if a country code is provided
        ctx.say(format!(
            "{}'s country rank in {}: {}",
            user.username,
            country_code,
            user.country_rank.unwrap_or(0)
        ))
        .await?;
    } else {
        // Fetch global rank
        ctx.say(format!(
            "{}'s global rank: {}",
            user.username,
            user.global_rank.unwrap_or(0)
        ))
        .await?;
    }

    Ok(())
}
