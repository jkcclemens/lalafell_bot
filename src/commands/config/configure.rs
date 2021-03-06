use lalafell::commands::prelude::*;

use super::channel;
use super::server;

#[derive(BotCommand)]
pub struct ConfigureCommand;

#[derive(Debug, StructOpt)]
#[structopt(about = "Manage server and channel settings.")]
pub enum Params {
  #[structopt(name = "channel", about = "Manage channel settings")]
  #[structopt(template = ::lalafell::commands::TEMPLATE)]
  #[structopt(setting = ::structopt::clap::AppSettings::ArgRequiredElseHelp)]
  Channel(channel::Params),

  #[structopt(name = "server", about = "Manage server settings")]
  #[structopt(template = ::lalafell::commands::TEMPLATE)]
  #[structopt(setting = ::structopt::clap::AppSettings::ArgRequiredElseHelp)]
  Server(server::Params)
}

impl HasParams for ConfigureCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for ConfigureCommand {
  fn run(&self, ctx: &Context, message: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    struct SubCommands {
      channel: channel::ChannelCommand,
      server: server::ServerCommand
    }

    const SUBCOMMANDS: SubCommands = SubCommands {
      channel: channel::ChannelCommand,
      server: server::ServerCommand
    };

    let params = self.params("config", params)?;

    match params {
      Params::Channel(p) => SUBCOMMANDS.channel.run(ctx, message.author.id, guild, p),
      Params::Server(p) => SUBCOMMANDS.server.run(ctx, message.author.id, guild, p)
    }
  }
}
