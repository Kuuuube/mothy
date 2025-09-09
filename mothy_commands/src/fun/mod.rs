pub mod hug;

#[must_use]
pub fn commands() -> Vec<crate::Command> {
    {
        hug::commands()
            .into_iter()
            .collect()
    }
}
