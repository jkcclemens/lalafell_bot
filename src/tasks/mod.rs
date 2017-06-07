use LalafellBot;
use config::Task;
use error::Result;

use std::sync::Arc;
use std::thread;

pub trait FromConfig {
  fn from_config(task: &Task) -> Result<Self>
    where Self: Sized;
}

pub trait RunsTask {
  fn start(self, s: Arc<LalafellBot>);
}

pub mod delete_all_messages;
pub mod database_save;
pub mod autotag;

pub use self::delete_all_messages::DeleteAllMessagesTask;
pub use self::database_save::DatabaseSaveTask;
pub use self::autotag::AutoTagTask;

pub struct TaskManager {
  bot: Arc<LalafellBot>
}

impl TaskManager {
  pub fn new(bot: Arc<LalafellBot>) -> TaskManager {
    TaskManager {
      bot: bot
    }
  }

  pub fn start_from_config(&self, task: &Task) -> Result<()> {
    match task.name.to_lowercase().as_ref() {
      "delete_all_messages" => self.start_task(DeleteAllMessagesTask::from_config(task)?),
      _ => return Err(format!("no task named {}", task.name).into())
    }
    Ok(())
  }

  pub fn start_task<T: RunsTask + Send + 'static>(&self, task: T) {
    let s = self.bot.clone();
    thread::spawn(move || {
      task.start(s);
    });
  }
}
