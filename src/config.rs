use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct AttributesConfig {
  ar: Option<String>,
  cs: Option<String>,
  hp: Option<String>,
  od: Option<String>,
  bpm: Option<String>,
  length: Option<String>,
  difficulty: Option<String>,
  playcount: Option<String>,
  nsfw: bool,
  creator: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
  mode: String,
  status: String,
  attributes: AttributesConfig,
  search: Option<String>,
  pub video: bool,
}

impl Config {
  pub fn get_mode(&self) -> i8 {
    match self.mode.to_lowercase().as_str() {
      "osu" | "std" | "standard" => 0,
      "taiko" | "drums" => 1,
      "fruits" | "ctb" | "catch" => 2,
      "mania" => 3,
      _ => -1,
    }
  }

  pub fn get_status(&self) -> i8 {
    match self.status.to_lowercase().as_str() {
      "graveyard" => -2,
      "wip" => -1,
      "pending" => 0,
      "ranked" => 1,
      "approved" => 2,
      "qualified" => 3,
      "loved" => 4,
      _ => -3,
    }
  }

  pub fn get_querystring(&self, offset: u32) -> String {
    let mut params: Vec<(&str, &str)> = Vec::new();
    let mut attributes = Vec::new();

    {
      // TODO: I have NO FUCKING IDEA how to use nsfw option

      // if !self.attributes.nsfw {
      //   attributes.push("nsfw = False".to_owned());
      // }
      if let Some(value) = &self.attributes.ar {
        attributes.push(format!("beatmaps.ar{}", value))
      }
      if let Some(value) = &self.attributes.cs {
        attributes.push(format!("beatmaps.cs{}", value))
      }
      if let Some(value) = &self.attributes.hp {
        attributes.push(format!("beatmaps.hp{}", value))
      }
      if let Some(value) = &self.attributes.od {
        attributes.push(format!("beatmaps.od{}", value))
      }
      if let Some(value) = &self.attributes.bpm {
        attributes.push(format!("beatmaps.bpm{}", value))
      }
      if let Some(value) = &self.attributes.length {
        attributes.push(format!("beatmaps.hit_length{}", value))
      }
      if let Some(value) = &self.attributes.difficulty {
        attributes.push(format!("beatmaps.difficulty_rating{}", value))
      }
      if let Some(value) = &self.attributes.playcount {
        attributes.push(format!("play_count{}", value))
      }
      if let Some(value) = &self.attributes.creator {
        attributes.push(format!("creator=\"{}\"", value))
      }
    }

    let query = if let Some(query) = &self.search {
      format!("{}[{}]", query, attributes.join(" AND "))
    } else {
      format!("[{}]", attributes.join(" AND "))
    };
    params.push(("q", query.as_str()));

    let mode = self.get_mode().to_string();
    params.push(("mode", mode.as_str()));

    let status = self.get_status().to_string();
    params.push(("status", status.as_str()));

    params.push(("limit", "1000"));

    let offset = offset.to_string();
    params.push(("offset", offset.as_str()));

    querystring::stringify(params)
  }
}

impl Default for Config {
  fn default() -> Self {
    Self {
      mode: "all".to_owned(),
      status: "all".to_owned(),
      attributes: AttributesConfig {
        ar: None,
        cs: None,
        hp: None,
        od: None,
        bpm: None,
        length: None,
        difficulty: None,
        playcount: None,
        nsfw: true,
        creator: None
      },
      search: None,
      video: true
    }
  }
}