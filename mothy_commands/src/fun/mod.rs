pub mod hug;
pub mod james_score;

#[must_use]
pub fn commands() -> Vec<crate::Command> {
    {
        hug::commands()
            .into_iter()
            .chain(james_score::commands())
            .collect()
    }
}
