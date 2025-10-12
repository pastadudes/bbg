// tried vibe coding this geniunely the ai was so dumb how can someone vibe code and push fullstack apps????
// tho the structure was pretty good aand ehh lgtm
use moka::future::Cache;
use rosu_v2::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub type OsuResult<T> = Result<T, crate::Error>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsuUser {
    pub id: u32,
    pub username: String,
    pub country_code: String,
    pub pp: f32,
    pub global_rank: Option<u32>,
    pub country_rank: Option<u32>,
    pub accuracy: f32,
    pub play_count: u32,
    pub level: f32,
    pub avatar_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsuBeatmap {
    pub id: u32,
    pub artist: String,
    pub title: String,
    pub creator: String,
    pub version: String,
    pub stars: f32,
    pub bpm: f32,
    pub ar: f32,
    pub cs: f32,
    pub hp: f32,
    pub od: f32,
    pub max_combo: u32,
    #[serde(skip)]
    pub background_image: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsuScore {
    pub id: u64,
    pub score: u32,
    pub max_combo: u32,
    pub perfect: bool,
    pub mods: String,
    pub pp: Option<f32>,
    pub rank: String,
    pub accuracy: f32,
    pub user: OsuUser,
    pub beatmap_id: Option<u32>,
    pub beatmap: Option<OsuBeatmap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatmapScores {
    pub beatmap: OsuBeatmap,
    pub scores: Vec<OsuScore>,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum UserIdentifier {
    Id(u32),
    Username(String),
}

#[cfg(feature = "discord")]
#[derive(Debug, Clone, poise::ChoiceParameter)]
pub enum ScoreType {
    #[name = "best scores"]
    Best,
    #[name = "recent scores"]
    Recent,
    #[name = "firsts"]
    Firsts,
}

#[cfg(not(feature = "discord"))]
#[derive(Debug, Clone)]
pub enum ScoreType {
    Best,
    Recent,
    Firsts,
}

pub struct OsuClient {
    client: Osu,
    beatmap_cache: Arc<Cache<u32, OsuBeatmap>>,
    user_cache: Arc<Cache<UserIdentifier, OsuUser>>,
}

impl OsuClient {
    pub async fn new(client_id: u64, client_secret: String) -> OsuResult<Self> {
        let client = Osu::new(client_id, client_secret)
            .await
            .map_err(|e| format!("osu API authentication failed: {}", e))?;
        let beatmap_cache = Arc::new(
            Cache::builder()
                .max_capacity(1000)
                .time_to_live(tokio::time::Duration::from_secs(3600))
                .build(),
        );
        let user_cache = Arc::new(
            Cache::builder()
                .max_capacity(10000) // "slighty" higher cache capacity
                .time_to_live(tokio::time::Duration::from_secs(3600))
                .build(),
        );

        Ok(Self {
            client,
            beatmap_cache,
            user_cache,
        })
    }

    pub async fn from_env() -> OsuResult<Self> {
        let client_id = std::env::var("osu_CLIENT_ID")
            .map_err(|_| "osu_CLIENT_ID environment variable not set")?
            .parse()
            .map_err(|_| "invalid osu_CLIENT_ID: must be a valid u64")?;

        let client_secret = std::env::var("osu_CLIENT_SECRET")
            .map_err(|_| "osu_CLIENT_SECRET environment variable not set")?;

        Self::new(client_id, client_secret).await
    }

    pub async fn get_user(&self, identifier: UserIdentifier) -> OsuResult<OsuUser> {
        let user = match identifier {
            UserIdentifier::Id(id) => self.client.user(id).await,
            UserIdentifier::Username(ref username) => self.client.user(username).await,
        }
        .map_err(|e| match e {
            rosu_v2::error::OsuError::NotFound => {
                format!("user not found: {:?}", identifier)
            }
            _ => format!("osu API error: {}", e),
        })?;

        let osu_user = OsuUser {
            id: user.user_id as u32,
            username: user.username.to_string(),
            country_code: user.country_code.to_string(),
            pp: user.statistics.as_ref().map(|s| s.pp).unwrap_or(0.0),
            global_rank: user.statistics.as_ref().and_then(|s| s.global_rank),
            country_rank: user.statistics.as_ref().and_then(|s| s.country_rank),
            accuracy: user.statistics.as_ref().map(|s| s.accuracy).unwrap_or(0.0),
            play_count: user.statistics.as_ref().map(|s| s.playcount).unwrap_or(0),
            level: user
                .statistics
                .as_ref()
                .map(|s| s.level.current as f32)
                .unwrap_or(0.0),
            avatar_url: user.avatar_url,
        };

        self.user_cache.insert(identifier, osu_user.clone()).await;
        Ok(osu_user)
    }

    pub async fn get_beatmap(&self, beatmap_id: u32) -> OsuResult<OsuBeatmap> {
        // get this shit from cache first
        if let Some(cached) = self.beatmap_cache.get(&beatmap_id).await {
            return Ok(cached);
        }

        let beatmap = self
            .client
            .beatmap()
            .map_id(beatmap_id)
            .await
            .map_err(|e| match e {
                rosu_v2::error::OsuError::NotFound => {
                    format!("beatmap not found: {}", beatmap_id)
                }
                _ => format!("osu API error: {}", e),
            })?;

        // get beatmapset for artist, title, and creator
        let (artist, title, creator) = if let Some(beatmapset) = &beatmap.mapset {
            (
                beatmapset.artist.to_string(),
                beatmapset.title.to_string(),
                beatmapset.creator_name.to_string(),
            )
        } else {
            ("???".to_string(), "???".to_string(), "???".to_string())
        };

        let image_request = reqwest::ClientBuilder::default()
            .user_agent("contact@pastaya.net if im being too spammy")
            .build()?
            .get(format!(
                "https://catboy.best/preview/background/{}",
                beatmap_id
            ))
            .send()
            .await?;

        let mut image = None;
        if image_request.status().is_success() {
            let bytes = image_request.bytes().await?;
            image = Some(bytes.to_vec());
        }

        let osu_beatmap = OsuBeatmap {
            id: beatmap.map_id as u32,
            artist,
            title,
            creator,
            version: beatmap.version,
            stars: beatmap.stars,
            bpm: beatmap.bpm,
            ar: beatmap.ar,
            cs: beatmap.cs,
            hp: beatmap.hp,
            od: beatmap.od,
            max_combo: beatmap.max_combo.unwrap_or(0) as u32,
            background_image: image,
        };
        // FIX ASAP: beatmap cache isn't working? also suspiciously low ram usage indicates that
        self.beatmap_cache
            .insert(beatmap_id, osu_beatmap.clone())
            .await;

        Ok(osu_beatmap)
    }

    pub async fn get_beatmap_scores(&self, beatmap_id: u32) -> OsuResult<BeatmapScores> {
        let beatmap = self.get_beatmap(beatmap_id).await?;
        let scores = self
            .client
            .beatmap_scores(beatmap_id)
            .await
            .map_err(|e| format!("Failed to fetch beatmap scores: {}", e))?;

        let mut converted_scores = Vec::new();

        for score in scores.scores {
            if let Some(user) = score.user {
                converted_scores.push(OsuScore {
                    id: score.id,
                    score: score.score as u32,
                    max_combo: score.max_combo as u32,
                    perfect: score.is_perfect_combo,
                    mods: score.mods.to_string(),
                    pp: score.pp,
                    rank: score.grade.to_string(),
                    accuracy: score.accuracy,
                    user: OsuUser {
                        id: user.user_id as u32,
                        username: user.username.to_string(),
                        country_code: user.country_code.to_string(),
                        pp: user.statistics.as_ref().map(|s| s.pp).unwrap_or(0.0),
                        global_rank: user.statistics.as_ref().and_then(|s| s.global_rank),
                        country_rank: user.statistics.as_ref().and_then(|s| s.country_rank),
                        accuracy: user.statistics.as_ref().map(|s| s.accuracy).unwrap_or(0.0),
                        play_count: user.statistics.as_ref().map(|s| s.playcount).unwrap_or(0),
                        level: user
                            .statistics
                            .as_ref()
                            .map(|s| s.level.current as f32)
                            .unwrap_or(0.0),
                        avatar_url: user.avatar_url,
                    },
                    beatmap_id: Some(beatmap.id),
                    beatmap: None,
                });
            }
        }

        Ok(BeatmapScores {
            beatmap,
            scores: converted_scores,
        })
    }

    pub async fn get_user_scores(
        &self,
        user_identifier: UserIdentifier,
        score_type: ScoreType,
        limit: Option<usize>,
    ) -> OsuResult<Vec<OsuScore>> {
        let user_id = match user_identifier {
            UserIdentifier::Id(id) => id,
            UserIdentifier::Username(username) => {
                let user = self
                    .client
                    .user(username)
                    .await
                    .map_err(|e| format!("User not found: {}", e))?;
                user.user_id as u32
            }
        };

        let scores = match score_type {
            ScoreType::Best => self.client.user_scores(user_id).best().await,
            ScoreType::Recent => self.client.user_scores(user_id).recent().await,
            ScoreType::Firsts => self.client.user_scores(user_id).firsts().await,
        }
        .map_err(|e| format!("Failed to fetch user scores: {}", e))?;

        let limit = limit.unwrap_or(10);
        let mut converted_scores = Vec::new();

        for score in scores.into_iter().take(limit) {
            if let Some(user) = score.user {
                converted_scores.push(OsuScore {
                    id: score.id,
                    score: score.score as u32,
                    max_combo: score.max_combo as u32,
                    perfect: score.is_perfect_combo,
                    mods: score.mods.to_string(),
                    pp: score.pp,
                    rank: score.grade.to_string(),
                    accuracy: score.accuracy,
                    user: OsuUser {
                        id: user.user_id as u32,
                        username: user.username.to_string(),
                        country_code: user.country_code.to_string(),
                        pp: user.statistics.as_ref().map(|s| s.pp).unwrap_or(0.0),
                        global_rank: user.statistics.as_ref().and_then(|s| s.global_rank),
                        country_rank: user.statistics.as_ref().and_then(|s| s.country_rank),
                        accuracy: user.statistics.as_ref().map(|s| s.accuracy).unwrap_or(0.0),
                        play_count: user.statistics.as_ref().map(|s| s.playcount).unwrap_or(0),
                        level: user
                            .statistics
                            .as_ref()
                            .map(|s| s.level.current as f32)
                            .unwrap_or(0.0),
                        avatar_url: user.avatar_url,
                    },
                    beatmap_id: Some(score.map_id), // not sure?
                    beatmap: None,
                });
            }
        }

        Ok(converted_scores)
    }

    // utility methods for common operations
    pub async fn search_user(&self, username: &str) -> OsuResult<OsuUser> {
        self.get_user(UserIdentifier::Username(username.to_string()))
            .await
    }

    pub async fn get_user_by_id(&self, user_id: u32) -> OsuResult<OsuUser> {
        self.get_user(UserIdentifier::Id(user_id)).await
    }

    pub async fn get_user_best_scores(
        &self,
        user_identifier: UserIdentifier,
        limit: Option<usize>,
    ) -> OsuResult<Vec<OsuScore>> {
        self.get_user_scores(user_identifier, ScoreType::Best, limit)
            .await
    }

    pub async fn get_user_recent_scores(
        &self,
        user_identifier: UserIdentifier,
        limit: Option<usize>,
    ) -> OsuResult<Vec<OsuScore>> {
        self.get_user_scores(user_identifier, ScoreType::Recent, limit)
            .await
    }

    pub async fn get_user_first_places(
        &self,
        user_identifier: UserIdentifier,
        limit: Option<usize>,
    ) -> OsuResult<Vec<OsuScore>> {
        self.get_user_scores(user_identifier, ScoreType::Firsts, limit)
            .await
    }
}

// convenience trait implementations
impl From<u32> for UserIdentifier {
    fn from(id: u32) -> Self {
        UserIdentifier::Id(id)
    }
}

impl From<String> for UserIdentifier {
    fn from(username: String) -> Self {
        UserIdentifier::Username(username)
    }
}

impl From<&str> for UserIdentifier {
    fn from(username: &str) -> Self {
        UserIdentifier::Username(username.to_string())
    }
}
