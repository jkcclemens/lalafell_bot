use crate::database::models::ToU64;

use diesel::prelude::*;

use lalafell::commands::prelude::*;
use lalafell::error::*;

use serenity::model::id::GuildId;

pub struct RemoveCommand;

#[derive(Debug, StructOpt)]
pub struct Params {
  #[structopt(help = "The ID of the DAM to delete")]
  id: i32
}

impl<'a> RemoveCommand {
  #[allow(clippy::needless_pass_by_value)]
  pub fn run(&self, guild: GuildId, params: Params) -> CommandResult<'a> {
    let affected = crate::bot::with_connection(|c| {
      use crate::database::schema::delete_all_messages::dsl;
      diesel::delete(
        dsl::delete_all_messages.filter(dsl::id.eq(params.id).and(dsl::server_id.eq(guild.to_u64())))
      )
        .execute(c)
    }).chain_err(|| "could not delete delete_all_messages")?;
    if affected > 0 {
      Ok(CommandSuccess::default())
    } else {
      Err("No delete all messages were deleted.".into())
    }
  }
}
