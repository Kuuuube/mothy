use crate::{Context, Error};

use mothy_core::structs::CommandData;
use poise::serenity_prelude as serenity;
use to_arraystring::ToArrayString;

use reqwest::Client as ReqwestClient;
use serde::Deserialize;

#[derive(Deserialize)]
struct Respon {
    list: Vec<Definition>,
}

#[derive(Deserialize)]
struct Definition {
    #[allow(clippy::struct_field_names)]
    definition: String,
    example: String,
    word: String,
    thumbs_up: u32,
    thumbs_down: u32,
    permalink: String,
}

// TODO: Add option to switch to a different definition

/// Get the definition of a term on Urban Dictionary
#[poise::command(
    prefix_command,
    slash_command,
    category = "Utility",
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn urban(
    ctx: Context<'_>,
    #[description = "Query to search the definition of"] term: String,
) -> Result<(), Error> {
    let command_data = &ctx.data().command_data;

    let reqwest = ReqwestClient::new();
    let response = reqwest
        .get("https://api.urbandictionary.com/v0/define")
        .query(&[("term", term.clone())])
        .send()
        .await?
        .json::<Respon>()
        .await?;
    if response.list.is_empty() {
        ctx.say(format!(
            "No definitions found for `{term}`. Try a different word."
        ))
        .await?;
        return Ok(());
    }
    let choice = &response.list[0];

    let thumbs_up = choice.thumbs_up.to_arraystring();
    let thumbs_down = choice.thumbs_down.to_arraystring();

    let embed = serenity::CreateEmbed::new()
        .title(&choice.word)
        .url(&choice.permalink)
        .description(format!(
            "**Definition:**\n{}\n\n **Example:**\n{}\n\n",
            inflate_links(&command_data, &choice.definition),
            inflate_links(&command_data, &choice.example)
        ))
        .field("👍", thumbs_up.as_str(), true)
        .field("👎", thumbs_down.as_str(), true);

    ctx.send(poise::CreateReply::new().embed(embed)).await?;

    Ok(())
}

pub fn inflate_links<'a>(command_data: &CommandData, text: &'a str) -> std::borrow::Cow<'a, str> {
    return command_data
        .urban_link_finder_regex
        .replace_all(text, &command_data.urban_link_replacement);
}

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [urban()]
}
