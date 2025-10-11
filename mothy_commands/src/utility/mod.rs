pub mod avatar;
pub mod colour;
pub mod info;
pub mod random;
pub mod urban;
pub mod patch_fix;

#[must_use]
pub fn commands() -> Vec<crate::Command> {
    {
        colour::commands()
            .into_iter()
            .chain(random::commands())
            .chain(info::commands())
            .chain(urban::commands())
            .chain(avatar::commands())
            .chain(patch_fix::commands())
            .collect()
    }
}
