use arrayvec::ArrayString;
use serde::{Deserialize, Serialize};
use serenity::all::{Colour, GenericChannelId, RoleColours, RoleId, RuleId};
use sqlx::types::time::Time;

pub(super) fn truncate_convert<const MAX_SIZE: usize>(mut s: String) -> ArrayString<MAX_SIZE> {
    if s.len() > MAX_SIZE {
        s.truncate(MAX_SIZE);
    }

    ArrayString::from(&s).expect("Already truncated to fit max size")
}

#[bool_to_bitflags::bool_to_bitflags]
#[derive(Default, Clone)]
pub struct GuildSettings {
    pub banned: bool,
    pub rejoined: bool,
    pub prefix: Option<ArrayString<6>>,
    pub features: GuildFeatures,
    pub regex_triggers: Vec<RegexTrigger>,
    pub regex_denylist: Vec<GlobalRegexDenylistChannel>,
    pub mod_roles: Vec<ModRole>,
    pub automod_rule_overrides: Vec<AutomodRuleOverrides>,
    pub sticky_role_settings: StickyRoleSettings,
    pub cotd_settings: Vec<CotdRoleSettings>,
    pub dm_activity_settings: DmActivitySettings,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Default, Hash)]
    pub struct GuildFeatures: u8 {
        const EXPRESSION_TRACKING = 1;
        const DM_ACTIVITY = 1 << 1;
        const AUTORESPONSE = 1 << 2;
        const AUTOMODERATION = 1 << 3;
        const STICKY_ROLES = 1 << 4;
        const COLOUR_OF_THE_DAY = 1 << 5;
    }
}

#[derive(Clone, Copy)]
pub struct DmActivitySettings {
    pub cooldown_seconds: u32,
    pub announce_channel_id: Option<GenericChannelId>,
    pub retention_days: Option<u8>,
}

impl Default for DmActivitySettings {
    fn default() -> Self {
        Self {
            cooldown_seconds: 3600,
            announce_channel_id: None,
            retention_days: None,
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct RawDmActivitySettings {
    pub cooldown_seconds: i32,
    pub announce_channel_id: Option<i64>,
    pub retention_days: Option<i16>,
}

impl From<RawDmActivitySettings> for DmActivitySettings {
    fn from(raw: RawDmActivitySettings) -> Self {
        DmActivitySettings {
            cooldown_seconds: raw.cooldown_seconds as u32,
            announce_channel_id: raw
                .announce_channel_id
                .map(|id| GenericChannelId::new(id as u64)),
            retention_days: raw.retention_days.map(|v| v as u8),
        }
    }
}

#[bool_to_bitflags::bool_to_bitflags(owning_setters)]
#[derive(Clone)]
pub struct RegexTrigger {
    pub id: u64,
    pub channel_id: Option<GenericChannelId>,
    pub pattern: Pattern,
    pub trigger_context: TriggerContext,
    pub trigger_metadata: TriggerMetadata,
    pub is_recursive: bool,
    pub is_enabled: bool,
}

#[derive(sqlx::FromRow)]
pub struct RawRegexTrigger {
    pub id: i64,
    pub channel_id: Option<i64>,
    pub pattern: String,
    pub trigger_context: TriggerContext,
    pub trigger_metadata: serde_json::Value,
    pub is_recursive: bool,
    pub is_enabled: bool,
    pub is_fancy: bool,
}

impl From<RawRegexTrigger> for RegexTrigger {
    fn from(raw: RawRegexTrigger) -> Self {
        let pattern = if raw.is_fancy {
            Pattern::Fancy(fancy_regex::Regex::new(&raw.pattern).expect("valid regex"))
        } else {
            Pattern::Simple(regex::Regex::new(&raw.pattern).expect("valid regex"))
        };

        let trigger_metadata: TriggerMetadata = serde_json::from_value(raw.trigger_metadata)
            .expect("failed to deserialize TriggerMetadata");

        RegexTrigger {
            id: raw.id as u64,
            channel_id: raw.channel_id.map(|id| GenericChannelId::new(id as u64)),
            pattern,
            trigger_context: raw.trigger_context,
            trigger_metadata,
            __generated_flags: RegexTriggerGeneratedFlags::empty(),
        }
        .set_is_enabled(raw.is_enabled)
        .set_is_recursive(raw.is_recursive)
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Default, Hash)]
    pub struct TriggerContext: u8 {
        const TEXT = 1;
        const OCR = 1 << 1;
    }
}

impl sqlx::Type<sqlx::Postgres> for TriggerContext {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for TriggerContext {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let raw = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        let bits = raw as u8;
        Ok(TriggerContext::from_bits_truncate(bits))
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TriggerMetadata {
    pub text: Option<String>,
    // emoji
    // moderation action
}

#[derive(Clone)]
pub struct GlobalRegexDenylistChannel {
    pub channel_id: GenericChannelId,
    pub is_recursive: bool,
}

impl From<RawGlobalRegexDenylistChannel> for GlobalRegexDenylistChannel {
    fn from(raw: RawGlobalRegexDenylistChannel) -> Self {
        GlobalRegexDenylistChannel {
            channel_id: GenericChannelId::new(raw.channel_id as u64),
            is_recursive: raw.is_recursive,
        }
    }
}

pub struct RawGlobalRegexDenylistChannel {
    pub channel_id: i64,
    pub is_recursive: bool,
}

#[derive(Clone, Copy)]
pub struct ModRole {
    pub role_id: RoleId,
    pub permissions: ModRolePermissions,
}

#[derive(sqlx::FromRow)]
pub struct RawModRole {
    pub role_id: i64,
    pub permissions: i64,
}

impl From<RawModRole> for ModRole {
    fn from(raw: RawModRole) -> Self {
        ModRole {
            role_id: RoleId::new(raw.role_id as u64),
            permissions: ModRolePermissions::from_bits_truncate(raw.permissions as u8),
        }
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Default, Hash)]
    pub struct ModRolePermissions: u8 {
        const EXIST = 1;
    }
}

#[derive(Clone)]
pub enum Pattern {
    Simple(regex::Regex),
    Fancy(fancy_regex::Regex),
}

#[derive(Clone)]
pub struct AutomodRuleOverrides {
    pub rule_id: RuleId,
    pub roles: Vec<(RoleId, bool)>,
}

#[derive(Default, Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "stickyrolemode")]
#[sqlx(rename_all = "lowercase")]
pub enum StickyRoleMode {
    #[default]
    None,
    Allowlist,
    Denylist,
}

#[derive(Default, Clone)]
pub struct StickyRoleSettings {
    pub allowlist_roles: Vec<RoleId>,
    pub denylist_roles: Vec<RoleId>,
    pub mode: StickyRoleMode,
    pub is_enabled: bool,
}

impl From<RawStickyRoleSettings> for StickyRoleSettings {
    fn from(raw: RawStickyRoleSettings) -> Self {
        StickyRoleSettings {
            allowlist_roles: raw
                .allowlist_roles
                .into_iter()
                .map(|id| RoleId::new(id as u64))
                .collect(),
            denylist_roles: raw
                .denylist_roles
                .into_iter()
                .map(|id| RoleId::new(id as u64))
                .collect(),
            mode: raw.mode,
            is_enabled: raw.is_enabled,
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct RawStickyRoleSettings {
    pub allowlist_roles: Vec<i64>,
    pub denylist_roles: Vec<i64>,
    pub mode: StickyRoleMode,
    pub is_enabled: bool,
}

#[bool_to_bitflags::bool_to_bitflags(owning_setters)]
#[derive(Clone)]
pub struct CotdRoleSettings {
    pub role_id: RoleId,
    pub is_enabled: bool,
    pub suffix_enabled: bool,
    pub colour_mode: ColourMode,
    pub icon_pairing_mode: IconPairingMode,
    pub colours: Vec<RoleColours>,
    pub icons: Vec<String>,
    pub svg_target_colour: Option<u32>,
    pub rotation_time: Time,
}

impl From<RawCotdRoleSettings> for CotdRoleSettings {
    fn from(raw: RawCotdRoleSettings) -> Self {
        let colours_vec: Vec<Vec<u32>> = serde_json::from_value(raw.colours).expect("valid json");

        let colours = colours_vec
            .into_iter()
            .map(|colours_vec| {
                let main = colours_vec
                    .first()
                    .cloned()
                    .expect("Expected at least one colour in the vector");

                RoleColours {
                    primary_colour: Colour::new(main),
                    secondary_colour: colours_vec.get(1).map(|c| Colour::new(*c)),
                    tertiary_colour: colours_vec.get(2).map(|c| Colour::new(*c)),
                }
            })
            .collect::<Vec<_>>();

        CotdRoleSettings {
            role_id: RoleId::new(raw.role_id as u64),
            colour_mode: raw.colour_mode,
            icon_pairing_mode: raw.icon_pairing_mode,
            colours,
            icons: raw.icons,
            svg_target_colour: raw.svg_target_colour.map(|v| v as u32),
            rotation_time: raw.rotation_time,
            __generated_flags: CotdRoleSettingsGeneratedFlags::empty(),
        }
        .set_is_enabled(raw.is_enabled)
        .set_suffix_enabled(raw.suffix_enabled)
    }
}

#[derive(sqlx::FromRow)]
pub struct RawCotdRoleSettings {
    pub role_id: i64,
    pub is_enabled: bool,
    pub suffix_enabled: bool,
    pub colour_mode: ColourMode,
    pub icon_pairing_mode: IconPairingMode,
    pub colours: serde_json::Value,
    pub icons: Vec<String>,
    pub svg_target_colour: Option<i32>,
    pub rotation_time: Time,
}

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "cotdiconpairingmode")]
#[sqlx(rename_all = "lowercase")]
pub enum IconPairingMode {
    Paired,
    Random,
}

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "cotdcolourmode")]
#[sqlx(rename_all = "lowercase")]
pub enum ColourMode {
    Random,
    Static,
}
