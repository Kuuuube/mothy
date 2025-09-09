use serde::{Deserialize, Serialize};

use crate::error::Error;
pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type Command = poise::Command<Data, Error>;

pub struct Data {
    pub time_started: std::time::Instant,
    pub has_started: std::sync::atomic::AtomicBool,
    pub database: crate::database::Database,
    pub james_scores: Vec<ScoresData>,
}

#[derive(Serialize, Deserialize)]
pub struct ScoresData {
    pub accuracy: f64,
    pub best_id: Option<i128>,
    pub created_at: String,
    pub id: i128,
    pub max_combo: i128,
    pub mode: String,
    pub mode_int: i128,
    pub mods: Vec<String>,
    pub passed: bool,
    pub perfect: bool,
    pub pp: Option<f64>,
    pub rank: String,
    pub replay: bool,
    pub score: i128,
    pub statistics: Statistics,
    pub user_id: i128,
    pub beatmap_id: i128,
    pub played_count: i128,
    pub beatmap: Beatmap,
}

#[derive(Serialize, Deserialize)]
pub enum ModeInt {
    Osu = 0,
    Taiko = 1,
    Catch = 2,
    Mania = 3,
}

#[derive(Serialize, Deserialize)]
pub struct Statistics {
    pub count_100: Option<i128>,
    pub count_300: Option<i128>,
    pub count_50: Option<i128>,
    pub count_geki: Option<i128>,
    pub count_katu: Option<i128>,
    pub count_miss: Option<i128>,
}

#[derive(Serialize, Deserialize)]
pub struct Beatmap {
    pub beatmapset_id: i128,
    pub difficulty_rating: f64,
    pub id: i128,
    pub mode: String,
    pub status: String,
    pub total_length: u64,
    pub user_id: i128,
    pub version: String,
    pub accuracy: f64,
    pub ar: f64,
    pub bpm: f64,
    pub convert: bool,
    pub count_circles: i128,
    pub count_sliders: i128,
    pub count_spinners: i128,
    pub cs: f64,
    pub drain: f64,
    pub hit_length: i128,
    pub is_scoreable: bool,
    pub last_updated: String,
    pub mode_int: i128,
    pub passcount: i128,
    pub playcount: i128,
    pub ranked: i8,
    pub url: String,
    pub checksum: String,
    pub beatmapset: BeatmapSet,
    pub current_user_playcount: i128,
    pub max_combo: i128,
}

#[derive(Serialize, Deserialize)]
pub struct BeatmapSet {
    pub artist: String,
    pub artist_unicode: String,
    pub covers: Covers,
    pub creator: String,
    pub favourite_count: i128,
    pub genre_id: i128,
    pub id: i128,
    pub language_id: i128,
    pub nsfw: bool,
    pub offset: i128,
    pub play_count: i128,
    pub preview_url: String,
    pub source: String,
    pub spotlight: bool,
    pub status: String,
    pub title: String,
    pub title_unicode: String,
    pub user_id: i128,
    pub video: bool,
    pub bpm: f64,
    pub can_be_hyped: bool,
    pub discussion_enabled: bool,
    pub discussion_locked: bool,
    pub is_scoreable: bool,
    pub last_updated: String,
    pub ranked: i8,
    pub ranked_date: String,
    pub rating: f64,
    pub storyboard: bool,
    pub submitted_date: String,
    pub tags:String,
}

#[derive(Serialize, Deserialize)]
pub struct Covers {
    pub cover: String,
    #[serde(rename = "cover@2x")]
    pub cover_2x: String,
    pub card: String,
    #[serde(rename = "card@2x")]
    pub card_2x: String,
    pub list: String,
    #[serde(rename = "list@2x")]
    pub list_2x: String,
    pub slimcover: String,
    #[serde(rename = "slimcover@2x")]
    pub slimcover_2x: String,
}
