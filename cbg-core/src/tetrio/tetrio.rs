use crate::Error;
use tetrio_api::models::users::summaries::tetra_league::LeagueSummary;
use tetrio_api::{
    http::clients::reqwest_client::InMemoryReqwestClient,
    models::{packet::Packet, users::user_info::UserInfo},
};
use tokio::time::Instant;

pub struct TetrioUser {
    pub username: String,
    pub id: String,
    pub xp: f64,
    pub role: String,
    pub league: Option<LeagueSummary>,
}

impl TetrioUser {
    pub async fn fetch(username: &str) -> Result<Self, Error> {
        let client = InMemoryReqwestClient::default();

        // Fetch user info
        let user_packet: Packet<UserInfo> = client
            .fetch_user_info(username)
            .await
            .map_err(|e| format!("Failed to fetch user info: {}", e))?;

        let mut user = Self::from_packet(user_packet)?;

        // Try to fetch league data if not already included
        if user.league.is_none() {
            user.league = Self::fetch_league(&user.id).await.ok();
        }

        Ok(user)
    }

    async fn fetch_league(user_id: &str) -> Result<LeagueSummary, Error> {
        let client = InMemoryReqwestClient::default();
        let packet: Packet<LeagueSummary> = client
            .fetch_user_league_summaries(user_id)
            .await
            .map_err(|e| format!("Failed to fetch league data: {}", e))?;

        match packet {
            Packet {
                data: Some(data), ..
            } => Ok(data),
            Packet { error, .. } => {
                if let Some(err) = error {
                    // Convert tetrio_api error to string
                    Err(format!("API error: {:?}", err).into())
                } else {
                    Err("Unknown error fetching league data".into())
                }
            }
        }
    }

    fn from_packet(packet: Packet<UserInfo>) -> Result<Self, Error> {
        match packet {
            Packet {
                data: Some(data), ..
            } => Ok(Self {
                username: data.username,
                id: data.id,
                xp: data.xp,
                role: format!("{:?}", data.role),
                league: None, // We'll fetch this separately
            }),
            Packet { error, .. } => {
                if let Some(err) = error {
                    // Convert tetrio_api error to string
                    Err(format!("API error!: {:?}", err).into())
                } else {
                    Err("unknown error from tetrio API!".into())
                }
            }
        }
    }

    pub fn level(&self) -> f64 {
        let xp = self.xp;
        let level =
            (xp / 500.0).powf(0.6) + (xp / (5000.0 + f64::max(0.0, xp - 4000000.0) / 5000.0)) + 1.0;
        level.trunc()
    }

    #[cfg(feature = "discord")]
    pub fn to_embed(&self) -> serenity::all::CreateEmbed {
        let mut embed = serenity::all::CreateEmbed::new()
            .title(&self.username)
            .field("id", &self.id, false)
            .field("xp", format!("{} XP", self.xp), false)
            .field("level", self.level().to_string(), true)
            .field("role", &self.role, false);

        if let Some(league) = &self.league {
            // handle league rank
            let rank_display = match &league.rank {
                Some(user_rank) => format!("{:?}", user_rank),
                None => "???".to_string(),
            };
            embed = embed.field("league rank", rank_display, true);

            // handle TR (Tetra Rating)
            let tr_display = match league.tr {
                Some(tr_value) => {
                    if tr_value >= 0.0 {
                        format!("{:.2}", tr_value)
                    } else {
                        "unranked".to_string()
                    }
                }
                None => "no data???".to_string(),
            };
            embed = embed.field("tr", tr_display, true);

            // handle GXE (basically how likely it is for someone to beat an average person)
            let gxe_display = match league.gxe {
                Some(gxe_value) => {
                    if gxe_value >= 0.0 {
                        format!("{:.1}%", gxe_value)
                    } else {
                        "???".to_string()
                    }
                }
                None => "no data".to_string(),
            };
            embed = embed.field("GXE", gxe_display, true);

            // self explanatory
            if let Some(apm_value) = league.apm {
                embed = embed.field("apm", format!("{:.1}", apm_value), true);
            }

            // you too bro
            if let Some(pps_value) = league.pps {
                embed = embed.field("pps", format!("{:.2}", pps_value), true);
            }
        }

        embed
    }
}

/// Use this struct for fetching server activity  
/// You can also use it to make a chart (KINDA similar to the one on ch.tetr.io)
/// Examples:
/// ```
/// // sorry but i was too lazy so i just used my bot, however its pretty easy to deduce
/// // You can help fix docs!
/// #[poise::command(slash_command, prefix_command, broadcast_typing)]
/// async fn activity(ctx: Context<'_>) -> Result<(), Error> {
///    let attachment = TetrioActivity::fetch().await?.create_chart()?; // notice how theres no "TetrioActivity::new().fetch"?
///    ctx.send(
///        poise::CreateReply::default().attachment(serenity::CreateAttachment::bytes(
///            attachment,
///            "tetrio_activity.png",
///        )),
///    )
///    .await?;
///    Ok(())
/// }
/// ```
pub struct TetrioActivity {
    /// Self explanatory (probably), DO NOT MODIFY THIS!
    pub data: Vec<f64>,
    /// you too man, DO NOT MODIFY THIS!  
    /// this however requires std (no idea why im mentioning that, my crate can NEVER work without std)  
    /// useful for cache invaildation if you like that  
    ///  
    /// why? just use moka bro please
    pub fetched_at: Instant,
}

impl TetrioActivity {
    /// obvious lol  
    /// DO NOT USE `TetrioActivity::new().fetch().await?` PLEASE!
    /// # errors
    /// - errors when tetrio feels like it probably
    pub async fn fetch() -> Result<Self, Error> {
        let client = InMemoryReqwestClient::default();
        let activity = client
            .fetch_general_activity()
            .await
            .map_err(|e| format!("failed to fetch activity: {}", e))?;

        match activity {
            Packet {
                data: Some(data), ..
            } => Ok(Self {
                data: data.activity.iter().map(|&x| x as f64).collect(),
                fetched_at: Instant::now(),
            }),
            Packet { error, .. } => {
                if let Some(err) = error {
                    Err(format!("API error: {:?}", err).into())
                } else {
                    Err("unknown error from tetrio API".into())
                }
            }
        }
    }

    /// makes a chart using plotters and returns raw bytes  
    /// Don't forget to `?`!  
    /// Example:  
    /// ```
    /// let image_bytes = TetrioActivity::fetch().await?.create_chart()?;
    /// // no idea what comes next bro :sob:
    /// ```
    /// # errors
    /// - errors out when you modify this method
    /// - failure to read the docs (bro i told you to use `TetrioActivity::fetch()` not whatever this is: `TetrioActivity::new().fetch()`)
    /// - a random bit switch
    pub fn create_chart(&self) -> Result<Vec<u8>, Error> {
        use image::ImageBuffer;
        use plotters::prelude::*;
        use plotters::style::colors::full_palette::ORANGE;
        use std::io::Cursor;
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
                self.data.iter().cloned().reduce(f64::min),
                self.data.iter().cloned().reduce(f64::max),
            ) {
                (Some(min), Some(max)) => (min, max),
                _ => return Err("no self.data".into()),
            };
            let pad = (max_val - min_val) * 0.1;
            let y_min = (min_val - pad).max(0.0);
            let y_max = max_val + pad;

            let mut chart = ChartBuilder::on(&root)
                .caption("tetrio server activity", ("sans-serif", 25))
                .margin(20)
                .x_label_area_size(40)
                .y_label_area_size(50)
                .build_cartesian_2d(0f64..self.data.len() as f64, y_min..y_max)?;

            chart
                .configure_mesh()
                .x_desc("time")
                .y_desc("players")
                .draw()?;

            chart.draw_series(LineSeries::new(
                self.data.iter().enumerate().map(|(i, &v)| (i as f64, v)),
                &ORANGE,
            ))?;

            root.present()?;
        }

        // 3. wrap raw rgb bytes in an `RgbImage` and encode to png
        let rgb_img: ImageBuffer<image::Rgb<u8>, _> =
            ImageBuffer::from_raw(W, H, raw).ok_or("buffer size mismatch")?;
        let mut png_bytes = Vec::new();
        rgb_img.write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)?;

        Ok(png_bytes)
    }
}
