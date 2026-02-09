use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serenity::all::{GenericChannelId, GuildId, RoleId};

use crate::error::Error;
pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type Command = poise::Command<Data, Error>;

pub struct Data {
    pub time_started: std::time::Instant,
    pub has_started: std::sync::atomic::AtomicBool,
    pub database: crate::database::Database,
    pub james_scores: Vec<ScoresData>,
    pub regex_filters: RegexFilters,
    pub config: MothyConfig,
}

pub struct RegexFilters {
    pub links_detector: regex::Regex,
    pub links_blacklist: Vec<regex::Regex>,
}

pub struct MothyConfig {
    pub events: Events,
    pub filters: Filters,
    pub logs: Logs,
}

impl MothyConfig {
    #[must_use]
    pub fn new() -> Self {
        MothyConfig {
            events: Events::default(),
            filters: Filters {
                // only allow filters to apply to kuuube server and test server
                filters_allowed_guilds: vec![902907712441040926.into(), 529423189860679702.into()],
                // regular role on kuuube server, test role on test server
                filter_bypass_roles: vec![1001489392457760828.into(), 1440516379840090345.into()],
            },
            logs: Logs {
                // kuuube server join logs channel, test server logs channel
                mothy_join_logs_channel: HashMap::from([
                    (902907712441040926.into(), 920370368135442442.into()),
                    (529423189860679702.into(), 894927450063138816.into()),
                ]),
                // kuuube server blacklist logs channel, test server logs channel
                mothy_blacklist_logs_channel: HashMap::from([
                    (902907712441040926.into(), 917776727801995304.into()),
                    (529423189860679702.into(), 894927450063138816.into()),
                ]),
                // kuuube server voice logs channel, test server logs channel
                mothy_voice_logs_channel: HashMap::from([
                    (902907712441040926.into(), 1443106469644996668.into()),
                    (529423189860679702.into(), 894927450063138816.into()),
                ]),
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Events {
    pub no_log_channels: Option<Vec<u64>>,
    pub no_log_users: Option<Vec<u64>>,
    pub guild_name_override: Option<HashMap<GuildId, String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Filters {
    pub filters_allowed_guilds: Vec<GuildId>,
    pub filter_bypass_roles: Vec<RoleId>,
}

pub struct Logs {
    pub mothy_join_logs_channel: HashMap<GuildId, GenericChannelId>,
    pub mothy_blacklist_logs_channel: HashMap<GuildId, GenericChannelId>,
    pub mothy_voice_logs_channel: HashMap<GuildId, GenericChannelId>,
}

#[derive(Serialize, Deserialize)]
pub struct ScoresData {
    pub classic_total_score: i128,
    pub preserve: bool,
    pub processed: bool,
    pub ranked: bool,
    pub maximum_statistics: MaximumStatistics,
    pub mods: Vec<Mod>,
    pub statistics: Statistics,
    pub beatmap_id: i128,
    pub best_id: Option<i128>,
    pub id: i128,
    pub rank: String,
    pub user_id: i128,
    pub accuracy: f64,
    pub ended_at: String,
    pub has_replay: bool,
    pub is_perfect_combo: bool,
    pub legacy_perfect: bool,
    pub legacy_score_id: Option<i128>,
    pub legacy_total_score: Option<i128>,
    pub max_combo: i128,
    pub passed: bool,
    pub pp: Option<f64>,
    pub ruleset_id: i128,
    pub started_at: Option<String>,
    pub total_score: i128,
    pub replay: bool,
    pub played_count: i128,
    pub beatmap: Beatmap,
}

#[derive(Serialize, Deserialize)]
pub struct Statistics {
    pub great: Option<i128>,
    pub ok: Option<i128>,
    pub meh: Option<i128>,
    pub miss: Option<i128>,
}

#[derive(Serialize, Deserialize)]
pub struct MaximumStatistics {
    pub great: Option<i128>,
    pub ignore_hit: Option<i128>,
    pub large_bonus: Option<i128>,
    pub small_bonus: Option<i128>,
    pub large_tick_hit: Option<i128>,
    pub slider_tail_hit: Option<i128>,
    pub legacy_combo_increase: Option<i128>,
}

#[derive(Serialize, Deserialize)]
pub struct Mod {
    pub acronym: String,
}

#[derive(Serialize, Deserialize)]
pub enum ModeInt {
    Osu = 0,
    Taiko = 1,
    Catch = 2,
    Mania = 3,
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
    pub tags: String,
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
