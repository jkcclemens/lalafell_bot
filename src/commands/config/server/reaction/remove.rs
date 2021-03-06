use crate::database::models::ToU64;

use diesel::prelude::*;

use lalafell::commands::prelude::*;
use lalafell::error::*;

use serenity::model::id::GuildId;

pub struct RemoveCommand;

#[derive(Debug, StructOpt)]
pub struct Params {
  #[structopt(help = "The ID of the reaction role to remove")]
  id: i32
}

impl<'a> RemoveCommand {
  #[allow(clippy::needless_pass_by_value)]
  pub fn run(&self, _: &Context, guild: GuildId, params: Params) -> CommandResult<'a> {
    let affected = crate::bot::with_connection(|c| {
      use crate::database::schema::reactions::dsl;
      diesel::delete(
        dsl::reactions.filter(dsl::id.eq(params.id).and(dsl::server_id.eq(guild.to_u64())))
      )
        .execute(c)
    }).chain_err(|| "could not delete reaction")?;
    if affected > 0 {
      Ok(CommandSuccess::default())
    } else {
      Err("No reactions were deleted.".into())
    }
  }
}
