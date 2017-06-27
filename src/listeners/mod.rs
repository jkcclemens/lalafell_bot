use bot::LalafellBot;
use config::Listener;

use lalafell::listeners::ReceivesEvents;

use error::*;

use std::sync::Arc;

pub mod debugger;
pub mod tag_instructions;
pub mod reaction_authorize;
pub mod timeouts;

pub use self::debugger::EventDebugger;
pub use self::tag_instructions::TagInstructions;
pub use self::reaction_authorize::ReactionAuthorize;
pub use self::timeouts::Timeouts;

pub struct ListenerManager;

impl ListenerManager {
  pub fn from_config(bot: Arc<LalafellBot>, listener: &Listener) -> Result<Box<ReceivesEvents + Send + Sync>> {
    let listener: Box<ReceivesEvents + Send + Sync> = match listener.name.to_lowercase().as_ref() {
      "tag_instructions" => box TagInstructions::new(bot.clone(), listener)?,
      "event_debugger" => box EventDebugger,
      "reaction_authorize" => box ReactionAuthorize::new(bot.clone(), listener)?,
      _ => bail!("no listener called {}", listener.name)
    };
    Ok(listener)
  }
}
