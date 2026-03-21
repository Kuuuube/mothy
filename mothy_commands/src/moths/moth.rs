use std::time::Duration;

use crate::{Context, Error, moths::embed_assemblers::*, moths::helpers::*};
use moth_filter::SpeciesData;
use poise::serenity_prelude as serenity;

use ::serenity::{
    all::{ComponentInteractionCollector, EmojiId, ReactionType},
    futures::StreamExt,
};
use rand::seq::IndexedRandom;

const MOTHS_PER_PAGE: usize = 10;
const MOTH_SEARCH_INTERACTION_TIMEOUT: u64 = 300; // interaction tokens are only valid for 15 minutes max, this should never exceed `900` (realistically a bit lower to have a bit of safety buffer)

const BUTTON_ID_PAGINATION_MODE: &str = "Pagination Mode";
const BUTTON_ID_PAGINATION_FIRST: &str = "Pagination First";
const BUTTON_ID_PAGINATION_BACK: &str = "Pagination Back";
const BUTTON_ID_PAGINATION_FORWARD: &str = "Pagination Forward";
const BUTTON_ID_PAGINATION_LAST: &str = "Pagination Last";
const BUTTON_ID_PAGINATION_GO_TO_PAGE: &str = "Pagination Go To Page";

const BUTTON_ID_SELECT_MODE: &str = "Select Mode";
const BUTTON_ID_SELECT_UP: &str = "Select Up";
const BUTTON_ID_SELECT_MOTH: &str = "Select Moth";
const BUTTON_ID_SELECT_DOWN: &str = "Select Down";

enum MothSearchMode {
    Pagination,
    Select,
    Moth,
}

/// Find a random moth
#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "Moths",
    user_cooldown = "10"
)]
pub async fn moth(ctx: Context<'_>) -> Result<(), Error> {
    // this command's response may take longer than 3 seconds of compute, defer to give us up to 15 minutes
    ctx.defer()
        .await
        .expect("moth command response defer fail, this shouldn't happen");

    let data = ctx.data();
    let moth = {
        let mut rng = rand::rng();
        data.moth_data.moth_data.choose(&mut rng).unwrap()
    };
    let embed = assemble_moth_embed(moth).await;
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// Search for a moth
#[poise::command(
    rename = "moth-search",
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "Moths",
    // This command may appear heavy on memory, it is not. All the moth data besides what is actively displayed is borrowed
    user_cooldown = "10"
)]
pub async fn moth_search(
    ctx: Context<'_>,
    #[description = "The superfamily to search for moths in"] superfamily: Option<String>,
    #[description = "The family to search for moths in"] family: Option<String>,
    #[description = "The subfamily to search for moths in"] subfamily: Option<String>,
    #[description = "The tribe to search for moths in"] tribe: Option<String>,
    #[description = "The subtribe to search for moths in"] subtribe: Option<String>,
    #[description = "The genus to search for moths in"] genus: Option<String>,
    #[description = "The specific name to search for moths in"] specific: Option<String>,
    #[description = "The subspecific name to search for moths in"] subspecific: Option<String>,
) -> Result<(), Error> {
    // this command's response may take longer than 3 seconds of compute, defer to give us up to 15 minutes
    ctx.defer()
        .await
        .expect("moth_search command response defer fail, this shouldn't happen");

    let data = ctx.data();
    let moth_data = &data.moth_data.moth_data;
    let moth_synonyms = &data.moth_data.moth_synonyms;
    let butterfly_blacklist = &data.moth_data.butterfly_blacklist;

    // ugly lepidoptera searching is not allowed (butteryflies)
    if is_butterfly(
        butterfly_blacklist,
        &superfamily,
        &family,
        &subfamily,
        &tribe,
        &subtribe,
        &genus,
        &specific,
        &subspecific,
    ) {
        let embed = serenity::CreateEmbed::default()
            .description("Attempted butterfly search detected. This incident will be reported.")
            .color(serenity::Colour::from_rgb(255, 0, 0));
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    // specific species search
    if let Some(genus_some) = &genus
        && let Some(specific_some) = &specific
    {
        let lowercase_scientific_name = assemble_scientific_name(
            &genus_some.to_lowercase(),
            &specific_some.to_lowercase(),
            subspecific.as_deref(),
        );
        let mut possible_subspecific_as_specific = Vec::new();
        let found_synonym_id = moth_synonyms.get(&lowercase_scientific_name);
        let found_moth = if let Some(found_synonym_id) = found_synonym_id {
            moth_data
                .iter()
                .find(|moth| &moth.catalogue_of_life_taxon_id == found_synonym_id)
        } else {
            moth_data.iter().find(|moth| {
                if !(moth.classification.genus.to_lowercase() == genus_some.to_lowercase()) {
                    return false;
                }
                if moth.classification.specific.to_lowercase() == specific_some.to_lowercase()
                    && moth.classification.subspecific
                        == subspecific.as_ref().map(|x| x.to_lowercase())
                {
                    return true;
                }

                // sometimes subspecies can be abbreviated to `Genus subspecific` rather than `Genus specific subspecific`
                // this can make it appear as if `Genus specific` has been written rather than a subspecies identifier
                // check if user's input `specific` matches moth's `subspecific`
                if let Some(moth_subspecific) = &moth.classification.subspecific
                    && moth_subspecific.to_lowercase() == specific_some.to_lowercase()
                {
                    possible_subspecific_as_specific.push(format!(
                        "{} {} {}",
                        moth.classification.genus, moth.classification.specific, moth_subspecific
                    ));
                }
                return false;
            })
        };

        if let Some(found_moth) = found_moth {
            let embed = assemble_moth_embed(found_moth).await;
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        } else {
            let mut capitalized_scientific_name = lowercase_scientific_name;
            capitalized_scientific_name
                .get_mut(0..1)
                .and_then(|x| Some(x.make_ascii_uppercase()));

            let mut embed_text = format!("Failed to find moth `{capitalized_scientific_name}`.");
            if possible_subspecific_as_specific.len() > 0 {
                let formatted_subspecific_as_specific = possible_subspecific_as_specific
                    .iter()
                    .map(|x| format!("`{x}`"))
                    .collect::<Vec<String>>()
                    .join("\n");
                embed_text = format!(
                    "{embed_text}\nFound similarly named subspecies:\n{formatted_subspecific_as_specific}"
                );
            }
            let embed = serenity::CreateEmbed::default().description(embed_text);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        return Ok(());
    }

    // wide search
    let mut moths_found: Vec<&SpeciesData> = moth_data
        .iter()
        .filter(|moth| {
            if !search_classification_valid(&superfamily, &moth.classification.superfamily)
                || !search_classification_valid(&family, &moth.classification.family)
                || !search_classification_valid(&subfamily, &moth.classification.subfamily)
                || !search_classification_valid(&tribe, &moth.classification.tribe)
                || !search_classification_valid(&subtribe, &moth.classification.subtribe)
                || !search_classification_valid(&genus, &Some(&moth.classification.genus))
                || !search_classification_valid(&specific, &Some(&moth.classification.specific))
                || !search_classification_valid(&subspecific, &moth.classification.subspecific)
            {
                return false;
            }
            return true;
        })
        .collect();

    let moth_count = moths_found.len();

    if moth_count == 0 {
        let embed = serenity::CreateEmbed::default().title("Search found 0 moths");
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    moths_found.sort_by(|a, b| {
        format!(
            "{} {} {}",
            a.classification.genus,
            a.classification.specific,
            a.classification.subspecific.as_deref().unwrap_or_default()
        )
        .cmp(&format!(
            "{} {} {}",
            b.classification.genus,
            b.classification.specific,
            a.classification.subspecific.as_deref().unwrap_or_default()
        ))
    });

    let mut current_mode = MothSearchMode::Pagination;
    let mut page_number = 0;
    let mut selected_moth = 0;
    let pagecount = (moth_count + MOTHS_PER_PAGE - 1) / MOTHS_PER_PAGE; // int division that rounds up
    let embed = assemble_paginated_moth_search_embed(
        &moths_found,
        moth_count,
        page_number,
        pagecount,
        None,
    );

    let bot_message = ctx
        .send(
            poise::CreateReply::default()
                .embed(embed.clone())
                .components(&[get_pagination_buttons(page_number, pagecount)]),
        )
        .await?;

    let mut interaction_collector = ComponentInteractionCollector::new(&ctx.serenity_context())
        .timeout(Duration::from_secs(MOTH_SEARCH_INTERACTION_TIMEOUT))
        .message_id(bot_message.message().await?.id)
        .stream();

    while let Some(interaction) = interaction_collector.next().await {
        // this interactions's response may take longer than 3 seconds of compute, defer to give us up to 15 minutes
        interaction
            .defer(ctx.http())
            .await
            .expect("Interaction defer fail, this shouldn't happen");
        match interaction.data.custom_id.clone().into_string().as_str() {
            BUTTON_ID_PAGINATION_MODE => {
                current_mode = MothSearchMode::Pagination;
            }
            BUTTON_ID_PAGINATION_FIRST => {
                if page_number == 0 {
                    continue;
                }
                page_number = 0;
            }
            BUTTON_ID_PAGINATION_BACK => {
                if page_number == 0 {
                    continue;
                }
                page_number -= 1;
            }
            BUTTON_ID_PAGINATION_FORWARD => {
                if page_number == pagecount - 1 {
                    continue;
                }
                page_number += 1;
            }
            BUTTON_ID_PAGINATION_LAST => {
                if page_number == pagecount - 1 {
                    continue;
                }
                page_number = pagecount - 1;
            }
            BUTTON_ID_PAGINATION_GO_TO_PAGE => {

            }
            BUTTON_ID_SELECT_MODE => {
                current_mode = MothSearchMode::Select;
                selected_moth = 0;
            }
            BUTTON_ID_SELECT_UP => {
                if selected_moth == 0 {
                    selected_moth =
                        (moth_count - page_number * MOTHS_PER_PAGE).min(MOTHS_PER_PAGE) - 1;
                } else {
                    selected_moth -= 1;
                }
            }
            BUTTON_ID_SELECT_MOTH => {
                current_mode = MothSearchMode::Moth;
            }
            BUTTON_ID_SELECT_DOWN => {
                if selected_moth
                    >= (moth_count - page_number * MOTHS_PER_PAGE).min(MOTHS_PER_PAGE) - 1
                {
                    selected_moth = 0;
                } else {
                    selected_moth += 1;
                }
            }
            _ => continue,
        };

        match current_mode {
            MothSearchMode::Pagination => {
                bot_message
                    .edit(
                        ctx,
                        poise::CreateReply::default()
                            .embed(assemble_paginated_moth_search_embed(
                                &moths_found,
                                moth_count,
                                page_number,
                                pagecount,
                                None,
                            ))
                            .components(&[get_pagination_buttons(page_number, pagecount)]),
                    )
                    .await?;
            }
            MothSearchMode::Select => {
                bot_message
                    .edit(
                        ctx,
                        poise::CreateReply::default()
                            .embed(assemble_paginated_moth_search_embed(
                                &moths_found,
                                moth_count,
                                page_number,
                                pagecount,
                                Some(selected_moth),
                            ))
                            .components(&[get_select_buttons()]),
                    )
                    .await?;
            }
            MothSearchMode::Moth => {
                let selected_moth_data = moths_found
                    .get(page_number * MOTHS_PER_PAGE + selected_moth)
                    .unwrap();
                bot_message
                    .edit(
                        ctx,
                        poise::CreateReply::default()
                            .embed(assemble_moth_embed(selected_moth_data).await)
                            .components(&[]),
                    )
                    .await?;

                return Ok(());
            }
        }
    }

    // edit out buttons after timeout
    // redoing the embed here is inefficient and unnecessary but it also doesn't really matter
    bot_message
        .edit(
            ctx,
            poise::CreateReply::default()
                .embed(assemble_paginated_moth_search_embed(
                    &moths_found,
                    moth_count,
                    page_number,
                    pagecount,
                    None,
                ))
                .components(&[]),
        )
        .await?;

    Ok(())
}

fn get_pagination_buttons<'a>(
    current_page: usize,
    last_page: usize,
) -> serenity::CreateComponent<'a> {
    // ⏮️
    let first_button = serenity::CreateButton::new(BUTTON_ID_PAGINATION_FIRST)
        .emoji(ReactionType::Custom {
            animated: false,
            id: EmojiId::new(1483966707809779813),
            name: None,
        })
        .disabled(current_page == 0);
    // ◀️
    let back_button = serenity::CreateButton::new(BUTTON_ID_PAGINATION_BACK)
        .emoji(ReactionType::Custom {
            animated: false,
            id: EmojiId::new(1483967178784112731),
            name: None,
        })
        .disabled(current_page == 0);
    // ⏺️
    let select_mode_button =
        serenity::CreateButton::new(BUTTON_ID_SELECT_MODE).emoji(ReactionType::Custom {
            animated: false,
            id: EmojiId::new(1483967184983167106),
            name: None,
        });
    // ▶️
    let forward_button = serenity::CreateButton::new(BUTTON_ID_PAGINATION_FORWARD)
        .emoji(ReactionType::Custom {
            animated: false,
            id: EmojiId::new(1483967180168101928),
            name: None,
        })
        .disabled(current_page == last_page - 1);
    // ⏭️
    let last_button = serenity::CreateButton::new(BUTTON_ID_PAGINATION_LAST)
        .emoji(ReactionType::Custom {
            animated: false,
            id: EmojiId::new(1483967181308821594),
            name: None,
        })
        .disabled(current_page == last_page - 1);
    // 🔢
    let goto_button = serenity::CreateButton::new(BUTTON_ID_PAGINATION_GO_TO_PAGE)
        .emoji(ReactionType::Custom {
            animated: false,
            id: EmojiId::new(1485048273789255841),
            name: None,
        });
    return serenity::CreateComponent::ActionRow(serenity::CreateActionRow::Buttons(
        vec![
            first_button,
            back_button,
            select_mode_button,
            forward_button,
            last_button,
            goto_button,
        ]
        .into(),
    ));
}

fn get_select_buttons<'a>() -> serenity::CreateComponent<'a> {
    // 🔼
    let back_button =
        serenity::CreateButton::new(BUTTON_ID_SELECT_UP).emoji(ReactionType::Custom {
            animated: false,
            id: EmojiId::new(1483967187625709608),
            name: None,
        });
    // ⏹️
    let select_mode_button =
        serenity::CreateButton::new(BUTTON_ID_SELECT_MOTH).emoji(ReactionType::Custom {
            animated: false,
            id: EmojiId::new(1483967186140790784),
            name: None,
        });
    // 🔽
    let forward_button =
        serenity::CreateButton::new(BUTTON_ID_SELECT_DOWN).emoji(ReactionType::Custom {
            animated: false,
            id: EmojiId::new(1483967183888453693),
            name: None,
        });
    // ↩️
    let back_to_pagination =
        serenity::CreateButton::new(BUTTON_ID_PAGINATION_MODE).emoji(ReactionType::Custom {
            animated: false,
            id: EmojiId::new(1483967182642745375),
            name: None,
        });
    return serenity::CreateComponent::ActionRow(serenity::CreateActionRow::Buttons(
        vec![
            back_button,
            select_mode_button,
            forward_button,
            back_to_pagination,
        ]
        .into(),
    ));
}

#[must_use]
pub fn commands() -> [crate::Command; 2] {
    [moth(), moth_search()]
}
