use crate::bot::is_administrator;
use crate::database::models::{Presence, NewPresence, PresenceKind};

use lalafell::error::*;
use lalafell::commands::prelude::*;

use serenity::model::gateway::Activity;

use diesel::prelude::*;

use itertools::Itertools;

#[derive(BotCommand)]
pub struct BotCommand;

impl HasParams for BotCommand {
  type Params = Params;
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Control the bot")]
pub struct Params {
  // FIXME: Use subcommands via clap
  #[structopt(help = "The subcommand")]
  subcommand: String,
  #[structopt(help = "Any arguments")]
  args: Vec<String>
}

impl<'a> Command<'a> for BotCommand {
  fn run(&self, ctx: &Context, message: &Message, params: &[&str]) -> CommandResult<'a> {
    if !is_administrator(&message.author)? {
      return Err(ExternalCommandFailure::default()
        .message(|e: &mut CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }
    let params = self.params_then("bot", params, |a| a.setting(structopt::clap::AppSettings::ArgRequiredElseHelp))?;
    let args = params.args;
    match params.subcommand.as_ref() {
      "presence" | "presences" => self.presence(ctx, &args),
      _ => Err("Invalid subcommand.".into())
    }
  }
}

impl BotCommand {
  fn presence<'a>(&self, ctx: &Context, args: &[String]) -> CommandResult<'a> {
    if args.is_empty() {
      return self.list_all_presences();
    }
    let subcommand = &args[0];
    let args = &args[1..];
    match subcommand.as_str() {
      "add" | "create" => self.add_presence(args),
      "remove" | "delete" => self.remove_presence(args),
      "change" | "set" => self.change_presence(ctx, args),
      "random" => self.random_presence(ctx),
      _ => Err("Invalid subcommand".into())
    }
  }

  fn list_all_presences<'a>(&self) -> CommandResult<'a> {
    let presences: Vec<Presence> = crate::bot::with_connection(|c| {
      use crate::database::schema::presences::dsl;
      dsl::presences.load(c)
    }).chain_err(|| "could not load presences")?;
    let strings = presences.iter()
      .map(|p| format!("{}. {} {}", p.id, PresenceKind::from_i16(p.kind).map(|x| x.to_string()).unwrap_or_else(|| "<invalid type>".to_string()), p.content))
      .join("\n");
    Ok(strings.into())
  }

  fn change_presence<'a>(&self, ctx: &Context, args: &[String]) -> CommandResult<'a> {
    if args.is_empty() {
      return Err("!bot presence change [playing/listening] [content]".into());
    }
    let name = args[1..].join(" ");
    let activity = match args[0].as_str() {
      "playing" => Activity::playing(&name),
      "listening" => Activity::listening(&name),
      _ => return Err("Invalid presence type.".into())
    };
    ctx.set_activity(activity);
    Ok(CommandSuccess::default())
  }

  fn random_presence<'a>(&self, ctx: &Context) -> CommandResult<'a> {
    match crate::tasks::random_presence::random_activity() {
      Some(g) => ctx.set_activity(g),
      None => return Err("No presences.".into())
    }
    Ok(CommandSuccess::default())
  }

  fn remove_presence<'a>(&self, args: &[String]) -> CommandResult<'a> {
    if args.is_empty() {
      return Err("!bot presences remove [id]".into());
    }
    let id: i32 = args[0].parse().map_err(|_| into!(CommandFailure, "Invalid ID."))?;
    let affected = crate::bot::with_connection(|c| {
      use crate::database::schema::presences::dsl;
      diesel::delete(dsl::presences.find(id)).execute(c)
    }).chain_err(|| "could not delete presence")?;
    if affected > 0 {
      Ok(CommandSuccess::default())
    } else {
      Err("No presence with that ID were found.".into())
    }
  }

  fn add_presence<'a>(&self, args: &[String]) -> CommandResult<'a> {
    if args.len() < 2 {
      return Err("!bot presences add [playing/listening] [content]".into());
    }
    let kind = match args[0].as_str() {
      "playing" => PresenceKind::Playing,
      "listening" => PresenceKind::Listening,
      _ => return Err("Invalid presence kind.".into())
    };
    let kind = kind as i16;
    let content = args[1..].join(" ");
    crate::bot::with_connection(|c| {
      use crate::database::schema::presences::dsl;
      diesel::insert_into(dsl::presences)
        .values(&NewPresence::new(kind, &content))
        .execute(c)
    }).chain_err(|| "could not add new presence")?;
    Ok(CommandSuccess::default())
  }
}
