// pub mod tag;
// pub mod autotag;

// pub use tag::*;
// pub use autotag::*;
/*
  fn search_tag(&self, who: UserId, on: &LiveServer, server: &str, character_name: &str) -> Result<Option<String>> {
    let mut params = HashMap::new();
    params.insert(String::from("one"), String::from("characters"));
    params.insert(String::from("strict"), String::from("on"));
    params.insert(String::from("server|et"), server.to_string());

    let res = self.xivdb.search(character_name.to_string(), params).chain_err(|| "could not query XIVDB")?;

    let search_chars = res.characters.unwrap().results;
    if search_chars.is_empty() {
      return Ok(Some(format!("Could not find any character by the name {} on {}.", character_name, server)));
    }

    let char_id = match search_chars[0]["id"].as_u64() {
      Some(u) => u,
      None => return Err("character ID was not a u64".into())
    };

    let name = match search_chars[0]["name"].as_str() {
      Some(s) => s,
      None => return Err("character name was not a string".into())
    };

    if name.to_lowercase() != character_name.to_lowercase() {
      return Ok(Some(format!("Could not find any character by the name {} on {}.", character_name, server)));
    }

    self.tag(who, on, char_id)
  }

  fn tag(&self, who: UserId, on: &LiveServer, char_id: u64) -> Result<Option<String>> {
    let character = self.xivdb.character(char_id).chain_err(|| "could not look up character")?;

    self.database.lock().unwrap().autotags.update_or_remove(AutotagUser::new(
      who.0,
      on.id.0,
      character.lodestone_id,
      &character.name,
      &character.server
    ));

    let roles = &on.roles;
    let mut add_roles = Vec::with_capacity(2);
    if let Some(r) = roles.iter().find(|x| x.name.to_lowercase() == character.data.race.to_lowercase()) {
      add_roles.push(r.id);
    }
    if let Some(r) = roles.iter().find(|x| x.name.to_lowercase() == character.data.gender.to_lowercase()) {
      add_roles.push(r.id);
    }
    if let Some(r) = roles.iter().find(|x| x.name.to_lowercase() == character.server.to_lowercase()) {
      add_roles.push(r.id);
    }

    self.discord.edit_member_roles(on.id, who, &add_roles).chain_err(|| "could not add roles")?;
    // cannot edit nickname of server owners
    self.discord.edit_member(on.id, who, |e| e.nickname(&character.name)).ok();
    Ok(None)
  }
*/
