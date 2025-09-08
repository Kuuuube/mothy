pub mod hug;

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [hug::hug()]
}
