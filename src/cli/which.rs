use eyre::Result;

use crate::cli::args::tool::{ToolArg, ToolArgParser};
use crate::config::Config;
use crate::dirs::SHIMS;

use crate::toolset::{Toolset, ToolsetBuilder};

/// Shows the path that a bin name points to
#[derive(Debug, clap::Args)]
#[clap(verbatim_doc_comment, after_long_help = AFTER_LONG_HELP)]
pub struct Which {
    /// The bin name to look up
    #[clap()]
    pub bin_name: String,

    /// Show the plugin name instead of the path
    #[clap(long, conflicts_with = "version")]
    pub plugin: bool,

    /// Show the version instead of the path
    #[clap(long, conflicts_with = "plugin")]
    pub version: bool,

    /// Use a specific tool@version
    /// e.g.: `rtx which npm --tool=node@20`
    #[clap(short, long, value_name = "TOOL@VERSION", value_parser = ToolArgParser, verbatim_doc_comment)]
    pub tool: Option<ToolArg>,
}

impl Which {
    pub fn run(self) -> Result<()> {
        let ts = self.get_toolset()?;

        match ts.which(&self.bin_name) {
            Some((p, tv)) => {
                if self.version {
                    rtxprintln!("{}", tv.version);
                } else if self.plugin {
                    rtxprintln!("{p}");
                } else {
                    let path = p.which(&tv, &self.bin_name)?;
                    rtxprintln!("{}", path.unwrap().display());
                }
                Ok(())
            }
            None => {
                if self.has_shim(&self.bin_name) {
                    bail!("{} is an rtx bin however it is not currently active. Use `rtx use` to activate it in this directory.", self.bin_name)
                } else {
                    bail!(
                        "{} is not an rtx bin. Perhaps you need to install it first.",
                        self.bin_name
                    )
                }
            }
        }
    }
    fn get_toolset(&self) -> Result<Toolset> {
        let config = Config::try_get()?;
        let mut tsb = ToolsetBuilder::new();
        if let Some(tool) = &self.tool {
            tsb = tsb.with_args(&[tool.clone()]);
        }
        let ts = tsb.build(&config)?;
        Ok(ts)
    }
    fn has_shim(&self, shim: &str) -> bool {
        SHIMS.join(shim).exists()
    }
}

static AFTER_LONG_HELP: &str = color_print::cstr!(
    r#"<bold><underline>Examples:</underline></bold>
  $ <bold>rtx which node</bold>
  /home/username/.local/share/rtx/installs/node/20.0.0/bin/node
  $ <bold>rtx which node --plugin</bold>
  node
  $ <bold>rtx which node --version</bold>
  20.0.0
"#
);

#[cfg(test)]
mod tests {

    #[test]
    fn test_which() {
        assert_cli!("use", "dummy@1.0.0");
        assert_cli_snapshot!("which", "dummy");
        assert_cli!("use", "dummy@ref:master");
        assert_cli!("uninstall", "dummy@1.0.0");
        assert_cli!("use", "--rm", "dummy");
    }

    #[test]
    fn test_which_plugin() {
        assert_cli!("use", "dummy@1.0.0");
        assert_cli_snapshot!("which", "--plugin", "dummy");
        assert_cli!("use", "dummy@ref:master");
        assert_cli!("uninstall", "dummy@1.0.0");
        assert_cli!("use", "--rm", "dummy");
    }

    #[test]
    fn test_which_version() {
        assert_cli!("use", "dummy@1.0.0");
        assert_cli_snapshot!("which", "--version", "dummy");
        assert_cli!("use", "dummy@ref:master");
        assert_cli!("uninstall", "dummy@1.0.0");
        assert_cli!("use", "--rm", "dummy");
    }

    #[test]
    fn test_which_tool() {
        assert_cli!("install", "dummy@1.0.1");
        assert_cli_snapshot!("which", "dummy", "--tool=dummy@1.0.1");
    }
}
