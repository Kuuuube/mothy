pub use mothy_core::{error::Error, structs::Data};
use serenity::all::{self as serenity, FullEvent};

mod messages;

pub struct Handler;

#[serenity::async_trait]
impl serenity::EventHandler for Handler {
    async fn dispatch(&self, ctx: &serenity::Context, event: &FullEvent) {
        if let Err(e) = event_handler(ctx, event).await {
            mothy_core::error::event_handler(ctx, e).await;
        }
    }
}

pub async fn event_handler(ctx: &serenity::Context, event: &FullEvent) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot, .. } => {
            let data = ctx.data_ref::<Data>();
            let shard_count = ctx.cache.shard_count();
            let is_last_shard = (ctx.shard_id.0 + 1) == shard_count.get();

            if is_last_shard
                && !data
                    .has_started
                    .swap(true, std::sync::atomic::Ordering::SeqCst)
            {
                println!("Logged in as {}", data_about_bot.user.tag());
            }
        }
        FullEvent::Message { new_message, .. } => {
            messages::on_message(ctx, new_message).await;
        }
        _ => (),
    }

    Ok(())
}
