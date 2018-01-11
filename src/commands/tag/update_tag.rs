use bot::BotEnv;
use commands::*;
use tasks::AutoTagTask;
use database::models::Tag;

use serenity::prelude::Mentionable;
use serenity::model::id::{GuildId, UserId};
use serenity::builder::CreateEmbed;

use lalafell::commands::prelude::*;
use lalafell::error::*;

use diesel::prelude::*;

pub struct UpdateTagCommand {
  env: Arc<BotEnv>
}

impl BotCommand for UpdateTagCommand {
  fn new(env: Arc<BotEnv>) -> Self {
    UpdateTagCommand { env }
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  who: Option<MentionOrId>
}

impl HasParams for UpdateTagCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for UpdateTagCommand {
  fn run(&self, _: &Context, message: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params("", params)?;
    let id = match params.who {
      Some(who) => {
        let member = guild.member(&message.author).chain_err(|| "could not get member")?;
        if !member.permissions().chain_err(|| "could not get permissions")?.manage_roles() {
          return Err(ExternalCommandFailure::default()
            .message(|e: CreateEmbed| e
              .title("Not enough permissions.")
              .description("You don't have enough permissions to update other people's tags."))
            .wrap());
        }
        *who
      },
      None => message.author.id
    };
    let tag: Option<Tag> = ::bot::CONNECTION.with(|c| {
      use database::schema::tags::dsl;
      dsl::tags
        .filter(dsl::user_id.eq(id.0.to_string()).and(dsl::server_id.eq(guild.0.to_string())))
        .first(c)
        .optional()
        .chain_err(|| "could not load tags")
    })?;
    let tag = match tag {
      Some(u) => u,
      None => return if id == message.author.id {
        Err("You are not set up with a tag. Use `!autotag` to tag yourself.".into())
      } else {
        Err(format!("{} is not set up with a tag.", id.mention()).into())
      }
    };
    match AutoTagTask::update_tag(self.env.as_ref(), UserId(*tag.user_id), GuildId(*tag.server_id), *tag.character_id) {
      Ok(Some(err)) => Err(err.into()),
      Err(e) => Err(e.into()),
      Ok(None) => Ok(CommandSuccess::default())
    }
  }
}
