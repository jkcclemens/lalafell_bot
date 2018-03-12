use lalafell::commands::prelude::*;

use lalafell::error::*;

use fflogs::{self, FfLogs};
use fflogs::net::{Server, Metric, Job};

use std::cmp::Ordering;
use std::str::FromStr;

pub struct FfLogsCommand {
  fflogs: FfLogs
}

impl ::commands::BotCommand for FfLogsCommand {
  fn new(env: Arc<::bot::BotEnv>) -> Self {
    FfLogsCommand {
      fflogs: FfLogs::new(&env.environment.fflogs_api_key)
    }
  }
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Display information about the character on FF Logs")]
pub struct Params {
  #[structopt(help = "The server the character is on")]
  server: String,
  #[structopt(help = "The character's first name")]
  first_name: String,
  #[structopt(help = "The character's last name")]
  last_name: String,
  #[structopt(
    short = "j",
    long = "job",
    help = "The job to look at",
    parse(try_from_str)
  )]
  job: Option<Job>
}

impl HasParams for FfLogsCommand {
  type Params = Params;
}

impl<'a> Command<'a> for FfLogsCommand {
  fn run(&self, _: &Context, _: &Message, params: &[&str]) -> CommandResult<'a> {
    let params = self.params_then("fflogs", params, |a| a.setting(::structopt::clap::AppSettings::ArgRequiredElseHelp))?;
    let server = match Server::from_str(&params.server) {
      Ok(s) => s,
      Err(_) => return Err("Invalid server.".into())
    };
    let name = format!("{} {}", params.first_name, params.last_name);

    let parses = self.fflogs.parses(
      &name,
      server,
      |x| x.metric(Metric::Dps)
    );

    let parses = match parses {
      Ok(r) => r,
      Err(fflogs::errors::Error::FfLogs(fflogs::net::FfLogsError { status, error })) => {
        return Err(format!("FF Logs didn't like that ({}): {}", status, error).into());
      },
      Err(e) => return Err(e).chain_err(|| "could not query fflogs")?
    };

    if parses.is_empty() {
      return Ok("No parses found.".into());
    }

    let first_spec = match parses[0].specs.get(0) {
      Some(s) => s,
      None => return Err("Somehow there was no first spec.".into())
    };
    let first_data = match first_spec.data.get(0) {
      Some(d) => d,
      None => return Err("Somehow there was no first data.".into())
    };

    let job = params.job.or_else(|| Job::from_str(&first_spec.spec).ok()).unwrap().to_string();
    let lower_job = job.to_lowercase().replace(" ", "");
    let name = &first_data.character_name;
    let id = first_data.character_id;

    let mut embed = CreateEmbed::default();

    embed = embed
      .title(&name)
      .url(format!("https://www.fflogs.com/character/id/{}", id))
      .field("Job", &job, true)
      .field("Server", server.as_ref(), true);

    let mut count = 0;

    for parse in &parses {
      let spec = match parse.specs.iter().find(|s| s.spec.to_lowercase() == lower_job) {
        Some(s) => s,
        None => continue
      };
      let data = match spec.data.iter().max_by(|a, b| a.historical_percent.partial_cmp(&b.historical_percent).unwrap_or(Ordering::Less)) {
        Some(d) => d,
        None => continue
      };

      count += 1;

      let url = format!("https://www.fflogs.com/reports/{}#fight={}", data.report_code, data.report_fight);

      let string = format!("[Link]({url}) – {dps:.2} DPS – {perc:.2} percentile out of {total_parses} parses",
        url = url,
        dps = data.persecondamount,
        perc = data.historical_percent,
        total_parses = data.historical_count
      );

      embed = embed.field(&parse.name, string, false);
    }

    if count == 0 {
      return Ok("No parses for that job.".into());
    }

    Ok(CommandSuccess::default().message(|_| embed))
  }
}
