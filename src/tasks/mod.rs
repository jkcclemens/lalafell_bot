use crate::bot::BotEnv;

use chrono::{Utc, DateTime};

use std::{
  sync::Arc,
  thread,
};

pub trait RunsTask {
  fn start(self, env: Arc<BotEnv>);
}

pub mod autotag;
pub mod delete_all_messages;
pub mod ephemeral_messages;
pub mod random_presence;
pub mod role_check;
pub mod tag_queue;
pub mod temporary_roles;
pub mod timeout_check;

pub use self::{
  autotag::AutoTagTask,
  delete_all_messages::DeleteAllMessagesTask,
  ephemeral_messages::EphemeralMessageTask,
  random_presence::RandomPresenceTask,
  role_check::RoleCheckTask,
  tag_queue::TagQueueTask,
  temporary_roles::TemporaryRolesTask,
  timeout_check::TimeoutCheckTask,
};

pub struct TaskManager {
  env: Arc<BotEnv>,
}

impl TaskManager {
  pub fn new(env: Arc<BotEnv>) -> Self {
    TaskManager { env }
  }

  pub fn start_task<T: RunsTask + Send + 'static>(&self, task: T) {
    let thread_env = Arc::clone(&self.env);
    thread::spawn(move || {
      task.start(thread_env);
    });
  }
}

pub struct Wait<T> {
  inner: T,
  now: DateTime<Utc>,
  last: i64,
}

impl<T, R> Wait<T>
  where T: Iterator<Item=(i64, R)>
{
  pub fn new(inner: T) -> Self {
    Self {
      inner,
      now: Utc::now(),
      last: 0,
    }
  }
}

impl<T, R> Iterator for Wait<T>
  where T: Iterator<Item=(i64, R)>
{
  type Item = (i64, R);

  fn next(&mut self) -> Option<Self::Item> {
    let item = self.inner.next()?;
    if item.0 <= self.now.timestamp() {
      return Some((0, item.1));
    }
    if self.last == 0 {
      self.last = self.now.timestamp();
    }
    let wait = item.0 - self.last;
    self.last += wait;
    Some((wait, item.1))
  }
}
