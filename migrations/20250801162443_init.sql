CREATE TABLE guilds (
    guild_id BIGINT PRIMARY KEY,
    banned BOOLEAN NOT NULL DEFAULT FALSE,
    rejoined BOOLEAN NOT NULL DEFAULT FALSE,
    prefix VARCHAR(6),
    feature_flags SMALLINT NOT NULL DEFAULT 0
);

CREATE TABLE users (
    user_id BIGINT PRIMARY KEY,
    is_bot_banned BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE stickers (
    sticker_id BIGINT PRIMARY KEY,
    sticker_name TEXT NOT NULL
);

CREATE TYPE CommandType AS ENUM (
    'prefix',
    'prefix_untracked',
    'prefix_edited',
    'application'
);

CREATE TABLE executed_commands (
    id SERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(user_id),
    channel_id INT NOT NULL,
    guild_id INT REFERENCES guilds(guild_id),
    command TEXT NOT NULL,
    command_type CommandType NOT NULL,
    executed_at TIMESTAMPTZ NOT NULL,
    executed_successfully BOOLEAN NOT NULL,
    error_text TEXT
);

CREATE TABLE sticker_usage (
    message_id BIGINT NOT NULL,
    sticker_id BIGINT NOT NULL REFERENCES stickers(sticker_id),
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id),
    user_id BIGINT NOT NULL REFERENCES users(user_id),
    channel_id BIGINT NOT NULL,
    used_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (message_id, user_id, sticker_id)
);

CREATE INDEX idx_sticker_usage_guild_user ON sticker_usage(guild_id, user_id);
CREATE INDEX idx_sticker_usage_guild_id ON sticker_usage(guild_id);
CREATE INDEX idx_sticker_usage_guild_sticker ON sticker_usage(guild_id, sticker_id);
CREATE INDEX idx_sticker_usage_channel_id ON sticker_usage(channel_id);

CREATE TABLE emotes (
    emote_id BIGINT PRIMARY KEY,
    emote_name TEXT NOT NULL,
    discord_id BIGINT UNIQUE
);

CREATE UNIQUE INDEX emote_name_discord_id_unique
    ON emotes (emote_name, discord_id);

CREATE UNIQUE INDEX emote_name_null_discord_id_unique
    ON emotes (emote_name)
    WHERE discord_id IS NULL;

CREATE TYPE EmoteUsageType AS ENUM ('message', 'reaction');

CREATE TABLE emote_usage (
    message_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id),
    channel_id BIGINT NOT NULL,
    emote_id BIGINT NOT NULL REFERENCES emotes(emote_id),
    user_id BIGINT NOT NULL REFERENCES users(user_id),
    used_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    usage_type EmoteUsageType NOT NULL,
    PRIMARY KEY (message_id, user_id, emote_id, usage_type)
);

CREATE INDEX idx_emote_usage_guild_id ON emote_usage(guild_id);
CREATE INDEX idx_emote_usage_guild_user ON emote_usage(guild_id, user_id);
CREATE INDEX idx_emote_usage_channel_id ON emote_usage(channel_id);

CREATE UNIQUE INDEX unique_user_message_emote_reaction
    ON emote_usage (message_id, user_id, emote_id)
    WHERE usage_type = 'reaction';

CREATE TABLE blocked_checked_emotes (
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id),
    emote_id BIGINT NOT NULL REFERENCES emotes(emote_id),
    PRIMARY KEY (guild_id, emote_id)
);

CREATE TABLE blocked_checked_stickers (
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id),
    sticker_id BIGINT NOT NULL REFERENCES stickers(sticker_id),
    PRIMARY KEY (guild_id, sticker_id)
);

CREATE TABLE dm_activity (
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id),
    user_id BIGINT NOT NULL REFERENCES users(user_id),
    last_announced TIMESTAMPTZ,
    until TIMESTAMPTZ,
    count SMALLINT,
    PRIMARY KEY (guild_id, user_id)
);

CREATE TABLE dm_activity_settings (
    guild_id BIGINT PRIMARY KEY REFERENCES guilds(guild_id) ON DELETE CASCADE,
    cooldown_seconds INT NOT NULL DEFAULT 3600,
    announce_channel_id BIGINT,
    retention_days SMALLINT DEFAULT NULL
);

CREATE TABLE role_snapshots (
    user_id BIGINT NOT NULL REFERENCES users(user_id),
    guild_id INT NOT NULL REFERENCES guilds(guild_id),
    roles BIGINT[],
    PRIMARY KEY (guild_id, user_id)
);


CREATE TABLE regex_triggers (
    id BIGSERIAL PRIMARY KEY,
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id),
    channel_id BIGINT,
    pattern TEXT NOT NULL,
    trigger_context SMALLINT NOT NULL DEFAULT 1,
    trigger_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    is_recursive BOOLEAN NOT NULL DEFAULT TRUE,
    is_fancy BOOLEAN NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE regex_global_denylist_channels (
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE,
    channel_id BIGINT NOT NULL,
    is_recursive BOOLEAN NOT NULL DEFAULT TRUE,
    PRIMARY KEY (guild_id, channel_id)
);


CREATE INDEX idx_regex_triggers_guild_id ON regex_triggers(guild_id);
CREATE INDEX idx_regex_triggers_channel_id ON regex_triggers(channel_id);
CREATE INDEX idx_regex_triggers_enabled_true ON regex_triggers(guild_id) WHERE is_enabled = TRUE;

CREATE TABLE autoresponse_settings (
    guild_id BIGINT PRIMARY KEY REFERENCES guilds(guild_id) ON DELETE CASCADE,
    allowed_roles BIGINT[] DEFAULT '{}',
    -- array of bitflags
    allowed_permissions BIGINT[] DEFAULT '{}',
    logging_channel_id BIGINT,
    log_changes BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE ocr_analytics (
    id BIGSERIAL PRIMARY KEY,
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE,
    channel_id BIGINT NOT NULL,
    word_count SMALLINT NOT NULL,
    line_count SMALLINT NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE mod_roles (
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE,
    role_id BIGINT NOT NULL PRIMARY KEY,
    permissions SMALLINT NOT NULL DEFAULT 0
);

CREATE INDEX idx_mod_roles_guild_id ON mod_roles(guild_id);

CREATE TABLE automod_rule_overrides (
    rule_id TEXT NOT NULL,
    role_id BIGINT NOT NULL REFERENCES mod_roles(role_id) ON DELETE CASCADE,
    is_allowed BOOLEAN NOT NULL DEFAULT TRUE,
    PRIMARY KEY (rule_id, role_id)
);

CREATE TYPE StickyRoleMode AS ENUM ('none', 'allowlist', 'denylist');

CREATE TABLE sticky_roles_settings (
    guild_id BIGINT PRIMARY KEY REFERENCES guilds(guild_id) ON DELETE CASCADE,
    allowlist_roles BIGINT[] NOT NULL DEFAULT '{}',
    denylist_roles BIGINT[] NOT NULL DEFAULT '{}',
    mode StickyRoleMode NOT NULL DEFAULT 'allowlist',
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE sticky_roles (
    user_id BIGINT NOT NULL REFERENCES users(user_id),
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id),
    roles BIGINT[] NOT NULL,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, guild_id)
);

CREATE TYPE COTDColourMode AS ENUM ('random', 'static');
CREATE TYPE COTDIconPairingMode AS ENUM ('paired', 'random');

CREATE TABLE cotd_role_settings (
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE,
    role_id BIGINT NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    suffix_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    colour_mode COTDColourMode NOT NULL DEFAULT 'random',
    icon_pairing_mode COTDIconPairingMode NOT NULL DEFAULT 'paired',
    colours JSONB NOT NULL DEFAULT '[]',
    icons TEXT[] NOT NULL DEFAULT '{}',
    svg_target_colour INT DEFAULT NULL,
    rotation_time TIME NOT NULL DEFAULT '00:00:00',
    PRIMARY KEY (guild_id, role_id)
);

CREATE TABLE cotd_colour_history (
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE,
    role_id BIGINT NOT NULL,
    colour INT NOT NULL,
    icon TEXT,
    assigned_at TIMESTAMPTZ DEFAULT NOW(),
    assigned_by BIGINT,
    PRIMARY KEY (guild_id, role_id, assigned_at)
);

CREATE INDEX idx_cotd_role_settings_guild_id ON cotd_role_settings(guild_id);
