#![expect(clippy::cast_possible_truncation)]

use crate::{Context, Error};
use image::{DynamicImage, ImageBuffer, Rgba};
use poise::serenity_prelude::{self as serenity, CreateAllowedMentions};

#[poise::command(
    prefix_command,
    slash_command,
    category = "Utility",
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    user_cooldown = "5"
)]
pub async fn hex(
    ctx: Context<'_>,
    #[description = "Space-separated hex colour codes"]
    #[rest]
    colours: String,
) -> Result<(), Error> {
    let colour_codes: Vec<&str> = colours
        .split([' ', ','])
        .filter(|s| !s.is_empty())
        .collect();

    let block_width = 160;
    let block_height = 160;

    let image_width = block_width * colour_codes.len() as u32;
    let image_height = block_height;

    let mut combined_image = ImageBuffer::new(image_width, image_height);

    let mut rgba_strings: Vec<String> = Default::default();

    for (i, colour) in colour_codes.iter().enumerate() {
        if let Ok(rgba) = hex_to_rgba(colour) {
            rgba_strings.push(format!(
                "#{} = rgba({})",
                colour,
                rgba.map(|x| x.to_string()).join(", ")
            ));
            for x in 0..block_width {
                for y in 0..block_height {
                    combined_image.put_pixel(i as u32 * block_width + x, y, Rgba(rgba));
                }
            }
        } else {
            let mentions = CreateAllowedMentions::new()
                .everyone(false)
                .all_roles(false)
                .all_users(false);

            ctx.send(
                poise::CreateReply::new()
                    .content(format!("Could not parse colour: {colour}"))
                    .allowed_mentions(mentions),
            )
            .await?;
            return Ok(());
        }
    }

    let bytes = {
        let mut buffer = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buffer);
        DynamicImage::ImageRgba8(combined_image)
            .write_to(&mut cursor, image::ImageFormat::Png)
            .unwrap();
        buffer
    };

    let attachment = serenity::CreateAttachment::bytes(bytes, "combined_colour.png");
    let content = rgba_strings.join(", ");
    ctx.send(
        poise::CreateReply::new()
            .content(content)
            .attachment(attachment),
    )
    .await?;

    Ok(())
}

fn hex_to_rgba(hex_color: &str) -> Result<[u8; 4], Error> {
    let replacements = ["#", "0x"];
    let hex_color = replacements
        .iter()
        .fold(hex_color.to_string(), |acc, x| acc.replacen(x, "", 1));

    let trimmed_hex_color = if hex_color.len() > 8 {
        &hex_color[0..8]
    } else if hex_color.len() == 3 || hex_color.len() == 4 {
        // 3 digit hex shorthand duplicates the digits to become 6 digits, 4 digit is the same but shorthand of 8 digits
        // FC0 <-> FFCC00
        &hex_color.chars().fold("".to_string(), |acc: String, x| {
            format!("{}{}{}", acc, x, x)
        })
    } else {
        &hex_color
    };

    let normalized_hex_color = if trimmed_hex_color.len() < 8 {
        format!("{trimmed_hex_color:0<6}FF")
    } else {
        trimmed_hex_color.to_string()
    };

    let r = u8::from_str_radix(&normalized_hex_color[0..2], 16)?;
    let g = u8::from_str_radix(&normalized_hex_color[2..4], 16)?;
    let b = u8::from_str_radix(&normalized_hex_color[4..6], 16)?;
    let a = u8::from_str_radix(&normalized_hex_color[6..8], 16)?;

    Ok([r, g, b, a])
}

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [hex()]
}
