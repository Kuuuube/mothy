pub mod guilds;

#[must_use]
pub fn commands() -> Vec<crate::Command> {
    guilds::commands().into_iter().collect()
}
