use std::{path::Path, fs::{self, File}, io::{self, BufReader}};

use discord::Discord;
use downloader::Downloader;
use serde::Deserialize;

use crate::{config::Config, log::Log};

mod config;
mod discord;
mod downloader;
mod log;
mod time;

#[tokio::main]
async fn main() {
  if !Path::new("./config.json").exists() {
    let config = serde_json::to_string_pretty(&Config::default()).unwrap();
    if let Err(_) = fs::write("./config.json", config) {
      Log::error("Unable to write config");
      return
    }
    Log::info("No config found. An example config was created.");
    return
  }

  let config: Config = {
    let reader = BufReader::new(File::open("./config.json").unwrap());
    let mut deserializer = serde_json::Deserializer::from_reader(reader);
    Config::deserialize(&mut deserializer).unwrap()
  };

  if !Path::new("./.data").exists() {
    fs::create_dir("./.data").expect("");
  }
  if !Path::new("./.data/completed").exists() {
    fs::write("./.data/completed", "").expect("");
  }
  if !Path::new("./.data/size").exists() {
    fs::write("./.data/size", "").expect("");
  }
  if !Path::new("./songs").exists() {
    fs::create_dir("./songs").expect("");
  }

  Log::info("Beatloader v0.727");
  Log::info("Loading saved maps");

  let binding = fs::read_to_string("./.data/completed")
    .unwrap();
  let mut cache: Vec<String> = binding
    .split(",")
    .map(|s| s.to_owned())
    .collect();
  
  {
    let songs = fs::read_dir("./songs").unwrap()
      .map(|res| res.map(|e| e.path()))
      .collect::<Result<Vec<_>, io::Error>>().unwrap();

    let len = songs.len();

    for song in songs {
      let filename = song.file_name().unwrap();
      let id = filename.to_str().unwrap().split(".osz").next().unwrap().to_owned();
      if cache.contains(&id) { continue }
      cache.push(id);
    }

    Log::success(format!("{} maps found", len).as_str())
  }

  let mut discord = Discord::new();

  discord.init();

  discord.start();

  let mut downloader = Downloader::new(cache, discord);

  let mut downloading = true;
  while downloading {
    downloading = downloader.crawl(&config).await;
  }

  Log::error("Reached end of downloads, terminating");
}
