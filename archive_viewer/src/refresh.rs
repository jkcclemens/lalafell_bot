use {MESSAGES, REFRESH_KEY};

use channel::{Archive, ArchiveServer};
use discordown;

use iron::prelude::*;
use iron::status;

use handlebars_iron::handlebars::*;

use url::Url;

use serde_json;

use discord::model::Message;

use std::path::PathBuf;
use std::fs::{self, File};
use std::collections::HashMap;

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

pub fn refresh(req: &mut Request) -> IronResult<Response> {
  let query: HashMap<_, _> = req.url.as_ref().query_pairs().collect();
  let key = query.get("key").map(ToOwned::to_owned).unwrap_or_else(Default::default);
  if key != *REFRESH_KEY {
    return Ok(Response::with(("Get outta here", status::Forbidden)));
  }
  _refresh();
  Ok(Response::with(("We gucci", status::Ok)))
}

fn find_id(part: &mut String, index: usize, start: usize) -> Option<(u64, usize)> {
  let end = part.find('>').unwrap_or_else(|| part.len() - 1);
  let id: u64 = match part[start + index..end].parse() {
    Ok(u) => u,
    Err(_) => return None
  };
  Some((id, end))
}

fn parse_user(server: &ArchiveServer, message: &Message, part: &mut String, index: usize, start: usize) -> bool {
  let (id, end) = match find_id(part, index, start) { Some(x) => x, None => return false };
  let name = match server.members.iter().find(|m| m.user.id.0 == id) {
    Some(member) => member.nick.as_ref().unwrap_or(&member.user.name),
    None => match message.mentions.iter().find(|m| m.id.0 == id) {
      Some(mention) => &mention.name,
      None => return false
    }
  };
  *part = format!("{}<span class=\"highlight\">@{}</span>{}",
    html_escape(&part[..start]),
    html_escape(name),
    html_escape(&part[end + 1..]));
  true
}

fn parse_user_mention(server: &ArchiveServer, message: &Message, part: &mut String, start: usize) -> bool {
  parse_user(server, message, part, 2, start)
}

fn parse_user_nick_mention(server: &ArchiveServer, message: &Message, part: &mut String, start: usize) -> bool {
  parse_user(server, message, part, 3, start)
}

fn parse_channel_mention(server: &ArchiveServer, part: &mut String, start: usize) -> bool {
let (id, end) = match find_id(part, 2, start) { Some(x) => x, None => return false };
  match server.channels.iter().find(|c| c.id.0 == id) {
    Some(channel) => {
      *part = format!("{}<span class=\"highlight\">#{}</span>{}",
        html_escape(&part[..start]),
        html_escape(&channel.name),
        html_escape(&part[end + 1..]));
      true
    },
    None => false
  }
}

fn parse_role_mention(server: &ArchiveServer, part: &mut String, start: usize) -> bool {
  let (id, end) = match find_id(part, 3, start) { Some(x) => x, None => return false };
  match server.roles.iter().find(|r| r.id.0 == id) {
    Some(role) => {
      let name = if role.name == "@everyone" { role.name.clone() } else { format!("@{}", role.name) };
      *part = format!("{}<span class=\"highlight\">{}</span>{}",
        html_escape(&part[..start]),
        html_escape(&name),
        html_escape(&part[end + 1..]));
      true
    },
    None => false
  }
}

fn parse_custom_emoji(part: &mut String, start: usize) -> bool {
  match part[start + 2..].find(':') {
    Some(index) => {
      let end = part[start + 2 + index..].find('>').map(|x| x + 2 + start + index).unwrap_or_else(|| part.len() - 1);
      let id: u64 = match part[start + 3 + index..end].parse() {
        Ok(u) => u,
        Err(_) => return false
      };
      *part = format!("{}<img class=\"emoji\" alt=\"{}\" src=\"https://cdn.discordapp.com/emojis/{}.png\"/>{}",
        html_escape(&part[..start]),
        html_escape(&part[start + 2..start + index]),
        id,
        html_escape(&part[end + 1..]));
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
      let escaped = if let Some(start_index) = part.find('<') {
        let tag = part[start_index..].to_owned();
        if tag.starts_with("<@!") {
          parse_user_nick_mention(&archive.server, message, part, start_index)
        } else if tag.starts_with("<@&") {
          parse_role_mention(&archive.server, part, start_index)
        } else if tag.starts_with("<@") {
          parse_user_mention(&archive.server, message, part, start_index)
        } else if tag.starts_with("<#") {
          parse_channel_mention(&archive.server, part, start_index)
        } else if tag.starts_with("<:") {
          parse_custom_emoji(part, start_index)
        } else {
          false
        }
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
    message.content = discordown::parse(&parts.join(" "));
  }
  let mut msgs = MESSAGES.write().unwrap();
  let server = msgs.entry(server_id).or_insert_with(Default::default);
  server.insert(channel_id, archive);
}
