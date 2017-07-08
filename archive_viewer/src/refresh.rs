use MESSAGES;

use channel::{Archive, ArchiveServer};

use iron::prelude::*;
use iron::status;

use handlebars_iron::handlebars::*;

use url::Url;

use serde_json;

use std::path::PathBuf;
use std::fs::{self, File};

pub fn _refresh() {
  let archives = fs::read_dir("archives").unwrap();
  for server in archives {
    let server = server.unwrap().path();
    if server.is_dir() {
      let server_id: u64 = server.file_stem().unwrap().to_string_lossy().parse().unwrap();
      for channel in fs::read_dir(server).unwrap() {
        let channel = channel.unwrap().path();
        let channel_id: u64 = channel.file_stem().unwrap().to_string_lossy().parse().unwrap();
        add_messages(channel, server_id, channel_id);
      }
    }
  }
}

pub fn refresh(_: &mut Request) -> IronResult<Response> {
  _refresh();
  Ok(Response::with(("We gucci", status::Ok)))
}

fn parse_user_mention(server: &ArchiveServer, part: &mut String) -> bool {
  let end = part.find('>').unwrap_or_else(|| part.len() - 1);
  let id: u64 = match part[2..end].parse() {
    Ok(u) => u,
    Err(_) => return false
  };
  match server.members.iter().find(|m| m.user.id.0 == id) {
    Some(member) => {
      let name = member.nick.as_ref().unwrap_or(&member.user.name);
      *part = format!("<span class=\"highlight\">@{}</span>", html_escape(name));
      true
    },
    None => false
  }
}

fn parse_user_nick_mention(server: &ArchiveServer, part: &mut String) -> bool {
  let end = part.find('>').unwrap_or_else(|| part.len() - 1);
  let id: u64 = match part[3..end].parse() {
    Ok(u) => u,
    Err(_) => return false
  };
  match server.members.iter().find(|m| m.user.id.0 == id) {
    Some(member) => {
      let name = member.nick.as_ref().unwrap_or(&member.user.name);
      *part = format!("<span class=\"highlight\">@{}</span>", html_escape(name));
      true
    },
    None => false
  }
}

fn parse_channel_mention(server: &ArchiveServer, part: &mut String) -> bool {
  let end = part.find('>').unwrap_or_else(|| part.len() - 1);
  let id: u64 = match part[2..end].parse() {
    Ok(u) => u,
    Err(_) => return false
  };
  match server.channels.iter().find(|c| c.id.0 == id) {
    Some(channel) => {
      *part = format!("<span class=\"highlight\">#{}</span>", html_escape(&channel.name));
      true
    },
    None => false
  }
}

fn parse_role_mention(server: &ArchiveServer, part: &mut String) -> bool {
  let end = part.find('>').unwrap_or_else(|| part.len() - 1);
  let id: u64 = match part[3..end].parse() {
    Ok(u) => u,
    Err(_) => return false
  };
  match server.roles.iter().find(|r| r.id.0 == id) {
    Some(role) => {
      let name = if role.name == "@everyone" { role.name.clone() } else { format!("@{}", role.name) };
      *part = format!("<span class=\"highlight\">{}</span>", html_escape(&name));
      true
    },
    None => false
  }
}

fn parse_custom_emoji(part: &mut String) -> bool {
  match part[2..].find(':') {
    Some(index) => {
      let end = part[2 + index..].find('>').map(|x| x + 2 + index).unwrap_or_else(|| part.len() - 1);
      let id: u64 = match part[3 + index..end].parse() {
        Ok(u) => u,
        Err(_) => return false
      };
      *part = format!("<img class=\"emoji\" alt=\"{}\" src=\"https://cdn.discordapp.com/emojis/{}.png\"/>", &part[2..index], id);
      true
    },
    None => false
  }
}

fn add_messages(channel: PathBuf, server_id: u64, channel_id: u64) {
  let f = File::open(channel).unwrap();
  let mut archive: Archive = serde_json::from_reader(f).unwrap();
  for message in &mut archive.messages {
    if let Some(member) = archive.server.members.iter().find(|mem| mem.user.id == message.author.id) {
      if let Some(ref nick) = member.nick {
        message.author.name = nick.clone();
      }
    }
    let mut parts: Vec<String> = message.content.split(' ').map(ToOwned::to_owned).collect();
    for part in &mut parts {
      let escaped = if part.starts_with("<@!") {
        parse_user_nick_mention(&archive.server, part)
      } else if part.starts_with("<@&") {
        parse_role_mention(&archive.server, part)
      } else if part.starts_with("<@") {
        parse_user_mention(&archive.server, part)
      } else if part.starts_with("<#") {
        parse_channel_mention(&archive.server, part)
      } else if part.starts_with("<:") {
        parse_custom_emoji(part)
      } else {
        false
      };

      if !escaped {
        *part = html_escape(part);
      }

      if let Ok(url) = Url::parse(part) {
        if url.has_host() {
          *part = format!("<a href=\"{url}\">{url}</a>", url=part)
        }
      }
    }
    message.content = parts.join(" ");
  }
  let mut msgs = MESSAGES.write().unwrap();
  let server = msgs.entry(server_id).or_insert_with(Default::default);
  server.insert(channel_id, archive);
}
