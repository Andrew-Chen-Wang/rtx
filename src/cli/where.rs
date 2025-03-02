use eyre::Result;

use crate::cli::args::tool::{ToolArg, ToolArgParser};
use crate::config::Config;
use crate::errors::Error::VersionNotInstalled;

use crate::toolset::ToolsetBuilder;

/// Display the installation path for a runtime
///
/// Must be installed.
#[derive(Debug, clap::Args)]
#[clap(verbatim_doc_comment, after_long_help = AFTER_LONG_HELP)]
pub struct Where {
    /// Tool(s) to look up
    /// e.g.: ruby@3
    /// if "@<PREFIX>" is specified, it will show the latest installed version
    /// that matches the prefix
    /// otherwise, it will show the current, active installed version
    #[clap(required = true, value_name = "TOOL@VERSION", value_parser = ToolArgParser, verbatim_doc_comment)]
    tool: ToolArg,

    /// the version prefix to use when querying the latest version
    /// same as the first argument after the "@"
    /// used for asdf compatibility
    #[clap(hide = true, verbatim_doc_comment)]
    asdf_version: Option<String>,
}

impl Where {
    pub fn run(self) -> Result<()> {
        let config = Config::try_get()?;
        let runtime = match self.tool.tvr {
            None => match self.asdf_version {
                Some(version) => self.tool.with_version(&version),
                None => {
                    let ts = ToolsetBuilder::new()
                        .with_args(&[self.tool.clone()])
                        .build(&config)?;
                    let v = ts
                        .versions
                        .get(&self.tool.plugin)
                        .and_then(|v| v.requests.first())
                        .map(|(r, _)| r.version());
                    self.tool.with_version(&v.unwrap_or(String::from("latest")))
                }
            },
            _ => self.tool,
        };

        let plugin = config.get_or_create_plugin(&runtime.plugin);

        match runtime
            .tvr
            .as_ref()
            .map(|tvr| tvr.resolve(plugin.as_ref(), Default::default(), false))
        {
            Some(Ok(tv)) if plugin.is_version_installed(&tv) => {
                rtxprintln!("{}", tv.install_path().to_string_lossy());
                Ok(())
            }
            _ => Err(VersionNotInstalled(
                runtime.plugin.to_string(),
                runtime.tvr.map(|tvr| tvr.version()).unwrap_or_default(),
            ))?,
        }
    }
}

static AFTER_LONG_HELP: &str = color_print::cstr!(
    r#"<bold><underline>Examples:</underline></bold>
  # Show the latest installed version of node
  # If it is is not installed, errors
  $ <bold>rtx where node@20</bold>
  /home/jdx/.local/share/rtx/installs/node/20.0.0

  # Show the current, active install directory of node
  # Errors if node is not referenced in any .tool-version file
  $ <bold>rtx where node</bold>
  /home/jdx/.local/share/rtx/installs/node/20.0.0
"#
);

#[cfg(test)]
mod tests {

    use crate::dirs;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn test_where() {
        assert_cli!("install");
        let stdout = assert_cli!("where", "tiny");
        assert_str_eq!(
            stdout.trim(),
            dirs::DATA.join("installs/tiny/3.1.0").to_string_lossy()
        );
    }

    #[test]
    fn test_where_asdf_style() {
        assert_cli!("install", "tiny@2", "tiny@3");
        assert_cli_snapshot!("where", "tiny", "2");
        assert_cli_snapshot!("where", "tiny", "3");
    }

    #[test]
    fn test_where_alias() {
        assert_cli!("install", "tiny@my/alias");
        let stdout = assert_cli!("where", "tiny@my/alias");
        assert_str_eq!(
            stdout.trim(),
            dirs::DATA.join("installs/tiny/3.0.1").to_string_lossy()
        );
        assert_cli!("uninstall", "tiny@my/alias");
    }

    #[test]
    fn test_where_not_found() {
        let err = assert_cli_err!("where", "tiny@1111");
        assert_display_snapshot!(err, @"tiny@1111 not installed");
    }
}
