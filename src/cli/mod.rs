use std::env::current_dir;
use std::path::PathBuf;

use anyhow::{bail, Context};
use structopt::StructOpt;

use crate::home::Home;
use crate::manifest::Manifest;
use crate::tool_alias::ToolAlias;
use crate::tool_name::ToolName;
use crate::tool_spec::ToolSpec;
use crate::tool_storage::ToolStorage;
use crate::trust::{TrustCache, TrustMode};

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(subcommand)]
    pub subcommand: Subcommand,
}

impl Args {
    pub fn run(self, home: &Home, tools: ToolStorage) -> anyhow::Result<()> {
        match self.subcommand {
            Subcommand::Init(sub) => sub.run(),
            Subcommand::Add(sub) => sub.run(tools),
            Subcommand::Install(sub) => sub.run(tools),
            Subcommand::Trust(sub) => sub.run(home),
            Subcommand::SelfInstall(sub) => sub.run(tools),

            Subcommand::List(_) => bail!("This command is not yet implemented."),
            Subcommand::Update(_) => bail!("This command is not yet implemented."),
            Subcommand::SelfUpdate(_) => bail!("This command is not yet implemented."),
        }
    }
}

#[derive(Debug, StructOpt)]
pub enum Subcommand {
    Init(InitSubcommand),
    List(ListSubcommand),
    Add(AddSubcommand),
    Update(UpdateSubcommand),
    Install(InstallSubcommand),
    Trust(TrustSubcommand),
    SelfUpdate(SelfUpdateSubcommand),
    SelfInstall(SelfInstallSubcommand),
}

/// Initialize a new Aftman manifest file.
#[derive(Debug, StructOpt)]
pub struct InitSubcommand {
    /// The folder to initialize the manifest file in. Defaults to the current
    /// directory.
    pub path: Option<PathBuf>,
}

impl InitSubcommand {
    pub fn run(self) -> anyhow::Result<()> {
        let path = match self.path {
            Some(v) => v,
            None => current_dir().context("Could not read current directory")?,
        };

        Manifest::init_local(&path)?;

        Ok(())
    }
}

/// Lists all existing tools managed by Aftman.
#[derive(Debug, StructOpt)]
pub struct ListSubcommand {}

/// Adds a new tool to Aftman and installs it.
#[derive(Debug, StructOpt)]
pub struct AddSubcommand {
    /// A tool spec describing where to get the tool and what version to
    /// install.
    pub tool_spec: ToolSpec,

    /// The name that will be used to run the tool.
    pub tool_alias: Option<ToolAlias>,

    /// Install this tool globally by adding it to ~/.aftman/aftman.toml instead
    /// of installing it to the nearest aftman.toml file.
    #[structopt(long)]
    pub global: bool,
}

impl AddSubcommand {
    pub fn run(self, tools: ToolStorage) -> anyhow::Result<()> {
        tools.add(&self.tool_spec, self.tool_alias.as_ref(), self.global)
    }
}

/// Updates one or more tools that are managed by Aftman.
///
/// Tools can be specified either by their alias or by their name.
///
/// If no tools are listed, Aftman will update all installed tools.
#[derive(Debug, StructOpt)]
pub struct UpdateSubcommand {
    /// One or more tools to update. If no tools are given, update all tools.
    pub aliases_or_specs: Vec<String>,

    /// Update this tool globally by adding it to ~/.aftman/aftman.toml instead
    /// of installing it to the nearest aftman.toml file that mentions it.
    #[structopt(long)]
    pub global: bool,

    /// Ignore semantic versioning and upgrade to the latest stable versions.
    #[structopt(long)]
    pub latest: bool,
}

/// Install all tools listed by Aftman files from the current directory.
#[derive(Debug, StructOpt)]
pub struct InstallSubcommand {
    /// Skip checking if these tools have been installed before. It is
    /// recommended to only run this on CI machines.
    #[structopt(long)]
    pub no_trust_check: bool,
}

impl InstallSubcommand {
    pub fn run(self, tools: ToolStorage) -> anyhow::Result<()> {
        let trust = if self.no_trust_check {
            TrustMode::NoCheck
        } else {
            TrustMode::Check
        };

        tools.install_all(trust)
    }
}

/// Mark the given tool name as being trusted.
#[derive(Debug, StructOpt)]
pub struct TrustSubcommand {
    /// The tool to mark as trusted.
    pub name: ToolName,
}

impl TrustSubcommand {
    pub fn run(self, home: &Home) -> anyhow::Result<()> {
        if TrustCache::add(home, self.name.clone())? {
            log::info!("Added {} to the set of trusted tools.", self.name);
        } else {
            log::info!("{} was already a trusted tool.", self.name);
        }

        Ok(())
    }
}

/// Update Aftman from the internet.
#[derive(Debug, StructOpt)]
pub struct SelfUpdateSubcommand {}

/// Install Aftman and update all references to it. Run this command if you've
/// just upgraded Aftman manually.
#[derive(Debug, StructOpt)]
pub struct SelfInstallSubcommand {}

impl SelfInstallSubcommand {
    pub fn run(self, tools: ToolStorage) -> anyhow::Result<()> {
        tools.update_links()?;

        if crate::system_path::add(&tools.bin_dir)? {
            log::info!(
                "Added ~/.aftman/bin to your PATH. Restart your terminal for this to take effect."
            );
        } else {
            log::debug!("Did not modify PATH.");
        }

        Ok(())
    }
}
