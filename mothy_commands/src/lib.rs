pub mod fun;
pub mod meta;
pub mod utility;

pub use mothy_core::{
    error::Error,
    structs::{Command, Context, Data},
};

#[must_use]
pub fn commands() -> Vec<crate::Command> {
    let mut commands: Vec<crate::Command> = meta::commands()
        .into_iter()
        .chain(fun::commands())
        .chain(utility::commands())
        .collect();

    if std::env::var("DEV_COMMANDS")
        .map(|e| e.parse::<bool>())
        .is_ok_and(Result::unwrap_or_default)
    {
        for command in &mut commands {
            command.name = std::borrow::Cow::Owned(format!("dev-{}", command.name));
        }
    }

    commands
}
