use crate::bot::BotEnv;
use crate::commands::*;
use crate::commands::tag::Tagger;

use ffxiv::World;

use lalafell::error::*;
use lalafell::commands::prelude::*;

use serenity::builder::CreateEmbed;

#[derive(BotCommand)]
pub struct TagCommand {
  env: Arc<BotEnv>
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Tag someone else as a FFXIV character")]
pub struct Params {
  #[structopt(help = "Who to tag")]
  who: MentionOrId,
  #[structopt(help = "The server the character is on, e.g. \"Adamantoise\"")]
  server: World,
  #[structopt(help = "The first name of the character")]
  first_name: String,
  #[structopt(help = "The last name of the character")]
  last_name: String
}

impl HasParams for TagCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for TagCommand {
  fn run(&self, ctx: &Context, message: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params_then("tag", params, |a| a.setting(structopt::clap::AppSettings::ArgRequiredElseHelp))?;
    let member = guild.member(ctx, &message.author).chain_err(|| "could not get member")?;
    if !member.permissions(&ctx).chain_err(|| "could not get permissions")?.manage_roles() {
      return Err(ExternalCommandFailure::default()
        .message(|e: &mut CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }

    let who = params.who;
    let ff_server = params.server;
    let name = format!("{} {}", params.first_name, params.last_name);

    match Tagger::search_tag(self.env.as_ref(), *who, guild, ff_server, &name, true) {
      Ok(Some(error)) => Err(ExternalCommandFailure::default()
        .message(move |e: &mut CreateEmbed| e.description(&error))
        .wrap()),
      Ok(None) => Ok(CommandSuccess::default()),
      Err(_) => Err(ExternalCommandFailure::default()
        .message(move |e: &mut CreateEmbed| e.description("There was an error while tagging. The user most likely does not exist or is not on the server."))
        .wrap())
    }
  }
}
