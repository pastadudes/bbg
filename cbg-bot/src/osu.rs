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

#[command(prefix_command, slash_command)]
async fn user(
    ctx: Context<'_>,
    #[description = "user identifier (ID or username)"] identifier: String,
) -> Result<(), Error> {
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

#[command(prefix_command, slash_command)]
async fn score(
    ctx: Context<'_>,
    #[description = "user identifier (ID or username)"] identifier: String,
    #[description = "score type (best, recent, firsts)"] score_type: Option<String>,
) -> Result<(), Error> {
    let user_identifier = identifier
        .parse::<u32>()
        .map_or_else(|_| UserIdentifier::Username(identifier), UserIdentifier::Id);

    let osu = OsuClient::from_env().await?;
    let score_type = match score_type.unwrap_or("best".to_string()).as_str() {
        "best" => ScoreType::Best,
        "recent" => ScoreType::Recent,
        "firsts" => ScoreType::Firsts,
        _ => {
            ctx.say("invalid score type. please use 'best', 'recent', or 'firsts'.")
                .await?;
            return Ok(());
        }
    };

    let scores = osu
        .get_user_scores(user_identifier, score_type, Some(5))
        .await?;

    let mut score_message = String::new();
    for score in scores {
        score_message.push_str(&format!(
            "beatmap: {}\nscore: {}\npp: {:.2}\nrank: {}\n\n", // test until embeds are made
            score
                .beatmap
                .as_ref()
                .map_or("???".to_string(), |b| b.title.clone()),
            score.score,
            score.pp.unwrap_or(0.0),
            score.rank
        ));
    }

    if score_message.is_empty() {
        score_message = "no scores found.".to_string();
    }

    ctx.say(score_message).await?;

    Ok(())
}

/// osu: fetches beatmap info by id
#[command(prefix_command, slash_command)]
async fn beatmap(
    ctx: Context<'_>,
    #[description = "beatmap ID"] beatmap_id: u32,
) -> Result<(), Error> {
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
