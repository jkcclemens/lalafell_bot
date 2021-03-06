use crate::{
  lodestone::Lodestone,
  database::models::{ToU64, Tag, Verification},
};

use lalafell::{
  commands::prelude::*,
  error::*,
};

use diesel::prelude::*;

use serenity::builder::CreateEmbed;

#[derive(BotCommand)]
pub struct VerifyCommand;

impl<'a> PublicChannelCommand<'a> for VerifyCommand {
  fn run(&self, ctx: &Context, message: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, _: &[&str]) -> CommandResult<'a> {
    let user: Option<Tag> = crate::bot::with_connection(|c| {
      use crate::database::schema::tags::dsl;
      dsl::tags
        .filter(dsl::user_id.eq(message.author.id.to_u64()).and(dsl::server_id.eq(guild.to_u64())))
        .first(c)
        .optional()
    }).chain_err(|| "could not load tags")?;
    let user = match user {
      Some(u) => u,
      None => return Err(ExternalCommandFailure::default()
        .message(|e: &mut CreateEmbed| e
          .title("Not tagged.")
          .description("Please tag yourself with an account before verifying it."))
        .wrap()),
    };
    let mut verification: Verification = crate::bot::with_connection(|c| {
      Verification::belonging_to(&user)
        .first(c)
        .optional()
    }).chain_err(|| "could not load verifications")?.unwrap_or_default();
    if verification.verified {
      return Err("You are already verified.".into());
    }
    let verification_string = match verification.verification_string {
      Some(ref v) => v,
      None => {
        let mut new_verification = verification.into_new(user.id);
        let msg = format!("Edit your Lodestone profile to contain `{}`.\nRerun the `!verify` command afterward.", new_verification.create_verification_string());
        crate::bot::with_connection(move |c| {
          use crate::database::schema::verifications;
          diesel::insert_into(verifications::table)
            .values(&new_verification)
            .execute(c)
        }).chain_err(|| "could not insert verification")?;
        message.author.direct_message(ctx, |c| c.embed(|e| e
          .title("Verification instructions")
          .description(&msg)
          .url("http://na.finalfantasyxiv.com/lodestone/my/setting/profile/"))).ok();
        return Ok(CommandSuccess::default());
      },
    };
    let profile = Lodestone::new().character_profile(*user.character_id)?;
    if profile.contains(verification_string) {
      let guild = guild.to_guild_cached(&ctx).chain_err(|| "could not find guild")?;

      verification.verified = true;
      crate::bot::with_connection(|c| verification.save_changes::<Verification>(c)).chain_err(|| "could not update verification")?;

      if let Some(r) = guild.read().roles.values().find(|x| x.name.to_lowercase() == "verified") {
        let mut member = guild.read().member(ctx, &message.author).chain_err(|| "could not get member for tagging")?;

        if !member.roles.contains(&r.id) {
          member.add_role(&ctx, r).chain_err(|| "could not add roles")?;
        }
      }
      let char_name = user.character.clone();
      let serv_name = user.server;
      Ok(CommandSuccess::default()
         .message(move |e: &mut CreateEmbed| e
           .title("Verified!")
           .description(&format!("You have successfully verified yourself as {} on {}.", char_name, serv_name))))
    } else {
      Err(ExternalCommandFailure::default().wrap())
    }
  }
}
