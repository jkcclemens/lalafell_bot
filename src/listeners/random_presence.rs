use serenity::client::{Context, EventHandler};
use serenity::model::gateway::Ready;

pub struct RandomPresenceListener;

impl EventHandler for RandomPresenceListener {
  fn ready(&self, ctx: Context, _: Ready) {
    if let Some(g) = ::tasks::random_presence::random_game() {
      ctx.shard.set_game(Some(g));
    }
  }
}
