use color_eyre::eyre::{eyre, Result};

use crate::config::config_file::ConfigFile;
use crate::config::Config;

/// Add/update a setting
///
/// This modifies the contents of ~/.config/rtx/config.toml
#[derive(Debug, clap::Args)]
#[clap(visible_aliases = ["add", "create"], after_long_help = AFTER_LONG_HELP, verbatim_doc_comment)]
pub struct SettingsSet {
    /// The setting to set
    #[clap()]
    pub setting: String,
    /// The value to set
    pub value: String,
}

impl SettingsSet {
    pub fn run(self) -> Result<()> {
        let value: toml_edit::Value = match self.setting.as_str() {
            "experimental" => parse_bool(&self.value)?,
            "always_keep_download" => parse_bool(&self.value)?,
            "always_keep_install" => parse_bool(&self.value)?,
            "legacy_version_file" => parse_bool(&self.value)?,
            "plugin_autoupdate_last_check_duration" => parse_i64(&self.value)?,
            "verbose" => parse_bool(&self.value)?,
            "asdf_compat" => parse_bool(&self.value)?,
            "jobs" => parse_i64(&self.value)?,
            "shorthands_file" => self.value.into(),
            "disable_default_shorthands" => parse_bool(&self.value)?,
            "raw" => parse_bool(&self.value)?,
            _ => return Err(eyre!("Unknown setting: {}", self.setting)),
        };

        let mut global_config = Config::try_get()?.global_config.clone();
        global_config.update_setting(&self.setting, value);
        global_config.save()
    }
}

fn parse_bool(value: &str) -> Result<toml_edit::Value> {
    match value {
        "true" => Ok(true.into()),
        "false" => Ok(false.into()),
        _ => Err(eyre!("{} must be true or false", value)),
    }
}

fn parse_i64(value: &str) -> Result<toml_edit::Value> {
    match value.parse::<i64>() {
        Ok(value) => Ok(value.into()),
        Err(_) => Err(eyre!("{} must be a number", value)),
    }
}

static AFTER_LONG_HELP: &str = color_print::cstr!(
    r#"<bold><underline>Examples:</underline></bold>
  $ <bold>rtx settings set legacy_version_file true</bold>
"#
);

#[cfg(test)]
pub mod tests {
    use crate::test::reset_config;

    #[test]
    fn test_settings_set() {
        reset_config();
        assert_cli!("settings", "set", "legacy_version_file", "false");
        assert_cli!("settings", "set", "always_keep_download", "true");
        assert_cli!(
            "settings",
            "set",
            "plugin_autoupdate_last_check_duration",
            "1"
        );

        assert_cli_snapshot!("settings");
        reset_config();
    }
}
