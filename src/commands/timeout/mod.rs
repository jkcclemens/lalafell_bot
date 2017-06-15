use bot::LalafellBot;
use error::*;

use discord::model::{RoleId, LiveServer, PermissionOverwrite, PermissionOverwriteType};
use discord::model::permissions::{self, Permissions};

pub mod timeout_command;
pub mod untimeout;

pub use self::timeout_command::TimeoutCommand;
pub use self::untimeout::UntimeoutCommand;

lazy_static! {
  static ref ROLE_PERMISSIONS: Permissions = {
    let mut perm = Permissions::empty();
    perm.insert(permissions::READ_MESSAGES);
    perm.insert(permissions::READ_HISTORY);
    perm.insert(permissions::VOICE_CONNECT);
    perm
  };
  static ref DENY_PERMISSIONS: Permissions = {
    let mut perm = Permissions::all();
    perm.remove(permissions::READ_MESSAGES);
    perm.remove(permissions::READ_HISTORY);
    perm.remove(permissions::VOICE_CONNECT);
    perm
  };
}

pub fn set_up_timeouts(bot: &LalafellBot, server: &LiveServer) -> Result<RoleId> {
  let role_name = match bot.config.bot.timeouts.timed_out_role {
    Some(ref r) => r,
    None => return Err("no timed-out role name".into())
  };
  let lower = role_name.to_lowercase();

  let role_id = match server.roles.iter().find(|r| r.name.to_lowercase() == lower) {
    Some(r) => r.id,
    None => bot.discord.create_role(server.id, Some(&role_name), Some(*ROLE_PERMISSIONS), None, None, None).chain_err(|| "could not create role")?.id
  };

  let target = PermissionOverwrite {
    kind: PermissionOverwriteType::Role(role_id),
    allow: Permissions::empty(),
    deny: *DENY_PERMISSIONS,
  };

  for channel in &server.channels {
    if channel.permission_overwrites.iter().any(|o| o.kind == target.kind) {
      continue;
    }
    if let Err(e) = bot.discord.create_permission(channel.id, target.clone()) {
      warn!("could not create permission overwrite for {}: {}", channel.id, e);
    }
  }
  Ok(role_id)
}
