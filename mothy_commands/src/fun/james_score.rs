use crate::{Context, Error};
use mothy_core::structs::Mod;
use poise::serenity_prelude as serenity;

use rand::seq::IndexedRandom;

/// Show a random osu! score from James
#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "Fun",
    user_cooldown = "4",
)]
pub async fn jamesscore(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let scores = &ctx.data().james_scores;
    let score = {
        let mut rng = rand::rng();
        scores.choose(&mut rng).unwrap()
    };

    let game_version = game_version(&score.mods);

    let title = format!("{} - {} [{}] [{}★]", score.beatmap.beatmapset.artist, score.beatmap.beatmapset.title, score.beatmap.version, score.beatmap.difficulty_rating);
    let description_line_0 = format!("**{}** • **{}** • **{:.2}%** • {} • {}", format_score_rank(score.rank.clone()), format_mods_string(&score.mods, &game_version), score.accuracy * 100.0, format_score_date(score.ended_at.clone()), game_version);
    let description_line_1 = format!("**{}pp** • **{}x**/{}x • {:.2}% • {}❌ • Playcount: {}", score.pp.unwrap_or_default(), score.max_combo, score.beatmap.max_combo, score.accuracy * 100.0, score.statistics.miss.unwrap_or_default(), score.played_count);
    let description_line_2 = format!("{} • CS: {} AR: {} OD: {} HP: {} • BPM: {}", format_duration_secs(score.beatmap.total_length), score.beatmap.cs, score.beatmap.ar, score.beatmap.accuracy, score.beatmap.drain, score.beatmap.bpm);
    let description_line_3 = format!("ScoreId: {} • MapId: {} • SetId: {}", score.id, score.beatmap.id, score.beatmap.beatmapset.id);
    let description = format!("{}\n{}\n{}\n{}", description_line_0, description_line_1, description_line_2, description_line_3);
    let embed = serenity::CreateEmbed::default().title(title).url(score.beatmap.url.clone()).description(description).thumbnail(score.beatmap.beatmapset.covers.list.clone());
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

fn game_version(mods: &Vec<Mod>) -> String {
    for osu_mod in mods {
        if osu_mod.acronym == "CL".to_string() {
            return "Stable".to_string();
        }
    }
    return "Lazer".to_string();
}

fn format_mods_string(mods: &Vec<Mod>, game_version: &String) -> String {
    if mods.len() == 0 || (game_version == "Stable" && mods.len() == 1) {
        return "NM".to_string();
    }
    let mut mods_string = "+".to_string();
    for osu_mod in mods {
        if osu_mod.acronym == "CL".to_string() {
            continue;
        }
        mods_string += &osu_mod.acronym;
    }
    return mods_string;
}

fn format_score_date(date: String) -> String {
    return date.replace("T", " ").replace("Z", "");
}

fn format_score_rank(rank: String) -> String {
    return rank.replace("H", "").replace("X", "SS");
}

fn format_duration_secs(duration: u64) -> String {
    let hours = duration / 3600;
    let minutes = (duration % 3600) / 60;
    let seconds = duration % 60;
    if duration > 3600 {
        return format!("{}:{}:{}", hours, minutes, seconds);
    }
    return format!("{}:{}", minutes, seconds);
}

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [jamesscore()]
}
