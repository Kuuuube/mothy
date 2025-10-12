pub mod avatar;
pub mod colour;
pub mod info;
pub mod patch_fix;
pub mod random;
mod udev;
pub mod urban;

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
            .chain(udev::commands())
            .collect()
    }
}
