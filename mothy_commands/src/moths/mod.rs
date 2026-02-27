pub mod api_callers;
pub mod moth;

#[must_use]
pub fn commands() -> Vec<crate::Command> {
    moth::commands().into_iter().collect()
}
