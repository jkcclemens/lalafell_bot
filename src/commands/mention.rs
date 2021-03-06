use lalafell::error::*;
use lalafell::commands::prelude::*;

use serenity::prelude::Mentionable;

use unicase::UniCase;

use std::sync::Arc;

#[derive(BotCommand)]
pub struct MentionCommand;

#[derive(Debug, StructOpt)]
#[structopt(about = "Mention roles that aren't mentionable")]
pub struct Params {
  #[structopt(help = "The role to mention")]
  role_name: String,
  #[structopt(help = "The message to send when mentioning the role")]
  message: Vec<String>
}

impl HasParams for MentionCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for MentionCommand {
  fn run(&self, ctx: &Context, msg: &Message, guild_id: GuildId, channel: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let member = guild_id.member(ctx, &msg.author).chain_err(|| "could not get member")?;
    if !member.permissions(&ctx).chain_err(|| "could not get permissions")?.mention_everyone() {
      return Err(ExternalCommandFailure::default()
        .message(|e: &mut CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }
    let params = self.params_then("mention", params, |a| a.setting(structopt::clap::AppSettings::ArgRequiredElseHelp))?;
    let guild = guild_id.to_guild_cached(&ctx).chain_err(|| "could not find guild")?;
    let uni_name = UniCase::new(&params.role_name);
    let role = match guild.read().roles.values().find(|r| UniCase::new(&r.name) == uni_name) {
      Some(r) => r.clone(),
      None => return Err("Could not find that role.".into())
    };
    let mentionable = role.mentionable;
    if !mentionable {
      guild_id.edit_role(&ctx, role.id, |r| r.mentionable(true)).ok();
    }
    msg.delete(ctx).ok();
    let p_message = if params.message.is_empty() {
      Default::default()
    } else {
      format!(" – {}", params.message.join(" "))
    };
    let message = format!("{}{}", role.mention(), p_message);
    channel.read().send_message(ctx, |m| m.content(&message)).ok();
    if !mentionable {
      guild_id.edit_role(&ctx, role.id, |r| r.mentionable(false)).ok();
    }
    Ok(CommandSuccess::default())
  }
}
