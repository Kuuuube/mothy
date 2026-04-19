use std::time::Duration;

use poise::serenity_prelude as serenity;

use crate::{Context, Error};
use ::serenity::{
    all::{
        ComponentInteractionCollector, CreateEmbed, CreateInputText, CreateInteractionResponse,
        CreateInteractionResponseMessage, CreateQuickModal, EmojiId, InputTextStyle, QuickModal,
        ReactionType,
    },
    futures::StreamExt,
};

const MOTH_SEARCH_INTERACTION_TIMEOUT: u64 = 300; // interaction tokens are only valid for 15 minutes max, this should never exceed `900` (realistically a bit lower to have a bit of safety buffer)

const BUTTON_ID_PAGINATION_MODE: &str = "Pagination Mode";
const BUTTON_ID_PAGINATION_FIRST: &str = "Pagination First";
const BUTTON_ID_PAGINATION_BACK: &str = "Pagination Back";
const BUTTON_ID_PAGINATION_FORWARD: &str = "Pagination Forward";
const BUTTON_ID_PAGINATION_LAST: &str = "Pagination Last";
const BUTTON_ID_PAGINATION_GO_TO_PAGE: &str = "Pagination Go To Page";
const MODAL_ID_PAGINATION_GO_TO_PAGE: &str = "Pagination Go To Page Modal";

const BUTTON_ID_SELECT_MODE: &str = "Select Mode";
const BUTTON_ID_SELECT_UP: &str = "Select Up";
const BUTTON_ID_SELECT_MOTH: &str = "Select Moth";
const BUTTON_ID_SELECT_DOWN: &str = "Select Down";

enum MothSearchMode {
    Pagination,
    Select,
    Moth,
}

pub async fn pagination_embed<
    'a,
    T,
    F1: Fn(&Vec<&T>, usize, usize, usize, Option<usize>) -> CreateEmbed<'a>,
    F2: AsyncFnOnce(&T) -> CreateEmbed<'a>,
>(
    ctx: Context<'_>,
    moths: &Vec<&T>,
    moths_per_page: usize,
    assemble_paginated_moth_search_embed: F1,
    assemble_moth_embed: F2,
) -> Result<(), Error> {
    let mut current_mode = MothSearchMode::Pagination;
    let mut page_number = 0;
    let mut selected_moth = 0;
    let moth_count = moths.len();
    let pagecount = moth_count.div_ceil(moths_per_page); // int division that rounds up
    let embed =
        assemble_paginated_moth_search_embed(moths, moth_count, page_number, pagecount, None);

    let bot_message = ctx
        .send(
            poise::CreateReply::default()
                .embed(embed)
                .components(&[get_pagination_buttons(page_number, pagecount)]),
        )
        .await?;

    let mut interaction_collector = ComponentInteractionCollector::new(ctx.serenity_context())
        .timeout(Duration::from_secs(MOTH_SEARCH_INTERACTION_TIMEOUT))
        .message_id(bot_message.message().await?.id)
        .stream();

    while let Some(interaction) = interaction_collector.next().await {
        // this interactions's response may take longer than 3 seconds of compute, defer to give us up to 15 minutes
        // exclude modal interactions
        if interaction.data.custom_id != BUTTON_ID_PAGINATION_GO_TO_PAGE {
            interaction
                .defer(ctx.http())
                .await
                .expect("Interaction defer fail, this shouldn't happen");
        }

        match interaction.data.custom_id.to_string().as_str() {
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
                let field = CreateInputText::new(
                    InputTextStyle::Short,
                    "Page Number",
                    MODAL_ID_PAGINATION_GO_TO_PAGE,
                );
                let modal = CreateQuickModal::new("Jump to Page").field(field);
                if let Ok(data_option) =
                    interaction.quick_modal(ctx.serenity_context(), modal).await
                    && let Some(data) = data_option
                {
                    if let Some(input_page_number_string) = data.inputs.first()
                        && let Ok(input_page_number) = input_page_number_string.parse::<usize>()
                        && input_page_number <= pagecount
                        && input_page_number >= 1
                    {
                        page_number = input_page_number - 1;
                        data.interaction
                            .create_response(ctx.http(), CreateInteractionResponse::Acknowledge)
                            .await?;
                    } else {
                        let message = CreateInteractionResponseMessage::new()
                            .content("Invalid page number")
                            .ephemeral(true);
                        data.interaction
                            .create_response(
                                ctx.http(),
                                CreateInteractionResponse::Message(message),
                            )
                            .await?;
                        continue;
                    }
                };
            }
            BUTTON_ID_SELECT_MODE => {
                current_mode = MothSearchMode::Select;
                selected_moth = 0;
            }
            BUTTON_ID_SELECT_UP => {
                if selected_moth == 0 {
                    selected_moth =
                        (moth_count - page_number * moths_per_page).min(moths_per_page) - 1;
                } else {
                    selected_moth -= 1;
                }
            }
            BUTTON_ID_SELECT_MOTH => {
                current_mode = MothSearchMode::Moth;
            }
            BUTTON_ID_SELECT_DOWN => {
                if selected_moth
                    >= (moth_count - page_number * moths_per_page).min(moths_per_page) - 1
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
                                moths,
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
                                moths,
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
                let selected_moth_data = moths
                    .get(page_number * moths_per_page + selected_moth)
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
                    moths,
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
    // 🔢
    let goto_button =
        serenity::CreateButton::new(BUTTON_ID_PAGINATION_GO_TO_PAGE).emoji(ReactionType::Custom {
            animated: false,
            id: EmojiId::new(1485048273789255841),
            name: None,
        });
    serenity::CreateComponent::ActionRow(serenity::CreateActionRow::Buttons(
        vec![back_button, select_mode_button, forward_button, goto_button].into(),
    ))
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
    serenity::CreateComponent::ActionRow(serenity::CreateActionRow::Buttons(
        vec![
            back_button,
            select_mode_button,
            forward_button,
            back_to_pagination,
        ]
        .into(),
    ))
}
