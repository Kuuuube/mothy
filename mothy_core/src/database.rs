use std::sync::Arc;

use dashmap::DashMap;
use serenity::all::GuildId;

use crate::database_models::{
    ColourMode, CotdRoleSettings, DmActivitySettings, GlobalRegexDenylistChannel, GuildFeatures,
    GuildSettings, GuildSettingsGeneratedFlags, IconPairingMode, ModRole, RawCotdRoleSettings,
    RawDmActivitySettings, RawGlobalRegexDenylistChannel, RawModRole, RawRegexTrigger,
    RawStickyRoleSettings, RegexTrigger, StickyRoleMode, StickyRoleSettings, TriggerContext,
    truncate_convert,
};

pub struct Database {
    /* pool: sqlx::PgPool, */
    pub guild_handler: GuildHandler,
}

impl Database {
    /// Gets an instance of the database.
    ///
    /// # Panics
    ///
    /// Will panic if `DATABASE_URL` is not set properly inside the envirnoment.
    pub async fn init() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set.");

        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect(&database_url)
            .await
            .expect("Failed to connect to database!");

        sqlx::migrate!("../migrations")
            .run(&pool)
            .await
            .expect("Could not run migrations.");

        Self {
            guild_handler: GuildHandler::new(pool),
            /*             pool, */
        }
    }
}

pub struct GuildHandler {
    pool: sqlx::PgPool,
    cache: DashMap<GuildId, Arc<GuildSettings>>,
}

impl GuildHandler {
    fn new(pool: sqlx::PgPool) -> Self {
        GuildHandler {
            pool,
            cache: DashMap::new(),
        }
    }

    pub async fn get(&self, guild_id: GuildId) -> anyhow::Result<Arc<GuildSettings>> {
        if let Some(cache_value) = self.cache.get(&guild_id) {
            return Ok(cache_value.clone());
        }

        let guild_or_default = self.get_(guild_id).await.map(Arc::new)?;
        self.cache.insert(guild_id, guild_or_default.clone());
        Ok(guild_or_default)
    }

    async fn get_(&self, guild_id: GuildId) -> anyhow::Result<GuildSettings> {
        let Ok(main_guild) = sqlx::query!(
            "SELECT prefix, feature_flags, rejoined, banned FROM guilds WHERE guild_id = $1",
            guild_id.get() as i64
        )
        .fetch_one(&self.pool)
        .await
        else {
            return Ok(GuildSettings::default());
        };

        let (dm_activity, regex_triggers, regex_denylist, sticky, cotd, mod_roles) = tokio::join!(
            get_dm_activity_settings(&self.pool, guild_id),
            get_regex_triggers_for_guild(&self.pool, guild_id),
            get_regex_denylists(&self.pool, guild_id),
            get_sticky_role_settings(&self.pool, guild_id),
            get_cotd_role_settings_for_guild(&self.pool, guild_id),
            get_mod_roles(&self.pool, guild_id)
        );

        let mut settings = GuildSettings {
            prefix: main_guild.prefix.map(truncate_convert),
            features: GuildFeatures::from_bits_truncate(main_guild.feature_flags as u8),
            regex_triggers: regex_triggers?,
            regex_denylist: regex_denylist?,
            mod_roles: mod_roles?,
            // TODO: implement
            automod_rule_overrides: vec![],
            sticky_role_settings: sticky?.unwrap_or_default(),
            cotd_settings: cotd?,
            dm_activity_settings: dm_activity?.unwrap_or_default(),
            __generated_flags: GuildSettingsGeneratedFlags::empty(),
        };

        settings.set_banned(main_guild.banned);
        settings.set_rejoined(main_guild.rejoined);

        Ok(settings)
    }
}

async fn get_dm_activity_settings(
    pool: &sqlx::PgPool,
    guild_id: GuildId,
) -> anyhow::Result<Option<DmActivitySettings>> {
    let raw = sqlx::query_as!(
        RawDmActivitySettings,
        r#"
        SELECT cooldown_seconds, announce_channel_id, retention_days
        FROM dm_activity_settings
        WHERE guild_id = $1
        "#,
        guild_id.get() as i64
    )
    .fetch_optional(pool)
    .await?;

    Ok(raw.map(DmActivitySettings::from))
}

async fn get_regex_triggers_for_guild(
    pool: &sqlx::PgPool,
    guild_id: GuildId,
) -> anyhow::Result<Vec<RegexTrigger>> {
    let raws = sqlx::query_as!(
        RawRegexTrigger,
        r#"
        SELECT
            id, channel_id, pattern,
            trigger_context as "trigger_context: TriggerContext",
            trigger_metadata,
            is_recursive, is_enabled, is_fancy
        FROM regex_triggers
        WHERE guild_id = $1
        "#,
        guild_id.get() as i64,
    )
    .fetch_all(pool)
    .await?;

    Ok(raws.into_iter().map(RegexTrigger::from).collect())
}

async fn get_sticky_role_settings(
    pool: &sqlx::PgPool,
    guild_id: GuildId,
) -> anyhow::Result<Option<StickyRoleSettings>> {
    let raw = sqlx::query_as!(
        RawStickyRoleSettings,
        r#"
        SELECT allowlist_roles, denylist_roles, mode as "mode: StickyRoleMode", is_enabled
        FROM sticky_roles_settings
        WHERE guild_id = $1
        "#,
        guild_id.get() as i64
    )
    .fetch_optional(pool)
    .await?;

    Ok(raw.map(StickyRoleSettings::from))
}

async fn get_cotd_role_settings_for_guild(
    pool: &sqlx::PgPool,
    guild_id: GuildId,
) -> anyhow::Result<Vec<CotdRoleSettings>> {
    let raws = sqlx::query_as!(
        RawCotdRoleSettings,
        r#"
        SELECT
            role_id,
            is_enabled,
            suffix_enabled,
            colour_mode as "colour_mode: ColourMode",
            icon_pairing_mode as "icon_pairing_mode: IconPairingMode",
            colours,
            icons,
            svg_target_colour,
            rotation_time
        FROM cotd_role_settings
        WHERE guild_id = $1
        "#,
        guild_id.get() as i64,
    )
    .fetch_all(pool)
    .await?;

    Ok(raws.into_iter().map(CotdRoleSettings::from).collect())
}

async fn get_mod_roles(pool: &sqlx::PgPool, guild_id: GuildId) -> anyhow::Result<Vec<ModRole>> {
    let raws = sqlx::query_as!(
        RawModRole,
        r#"
        SELECT
            role_id,
            permissions
        FROM mod_roles
        WHERE guild_id = $1
        "#,
        guild_id.get() as i64,
    )
    .fetch_all(pool)
    .await?;

    Ok(raws.into_iter().map(ModRole::from).collect())
}

async fn get_regex_denylists(
    pool: &sqlx::PgPool,
    guild_id: GuildId,
) -> anyhow::Result<Vec<GlobalRegexDenylistChannel>> {
    let raws = sqlx::query_as!(
        RawGlobalRegexDenylistChannel,
        r#"
        SELECT
            channel_id,
            is_recursive
        FROM regex_global_denylist_channels
        WHERE guild_id = $1
        "#,
        guild_id.get() as i64,
    )
    .fetch_all(pool)
    .await?;

    Ok(raws
        .into_iter()
        .map(GlobalRegexDenylistChannel::from)
        .collect())
}
