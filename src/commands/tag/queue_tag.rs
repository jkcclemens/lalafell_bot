use crate::commands::*;
use crate::database::models::NewTagQueue;

use diesel::prelude::*;

use ffxiv::World;

use lalafell::error::*;
use lalafell::commands::prelude::*;

use serenity::builder::CreateEmbed;

#[derive(BotCommand)]
pub struct QueueTagCommand;

#[derive(Debug, StructOpt)]
#[structopt(about = "Queue a tag to be done later")]
pub struct Params {
  #[structopt(help = "Who to queue the tag for")]
  who: MentionOrId,
  #[structopt(help = "The server the character is on, e.g. \"Adamantoise\"")]
  server: World,
  #[structopt(help = "The first name of the character")]
  first_name: String,
  #[structopt(help = "The last name of the character")]
  last_name: String
}

impl HasParams for QueueTagCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for QueueTagCommand {
  fn run(&self, ctx: &Context, message: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params_then("queuetag", params, |a| a.setting(structopt::clap::AppSettings::ArgRequiredElseHelp))?;
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

    let item = NewTagQueue::new(who.0, guild.0, ff_server.as_str(), &name);

    crate::bot::with_connection(|c| {
      use crate::database::schema::tag_queue::dsl;
      diesel::insert_into(dsl::tag_queue)
        .values(&item)
        .execute(c)
    }).chain_err(|| "could not insert tag queue")?;

    Ok(CommandSuccess::default())
  }
}
