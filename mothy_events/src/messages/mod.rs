pub async fn on_message(_: &serenity::all::Context, msg: &serenity::all::Message) {
    let Some(_) = msg.guild_id else { return };
}
