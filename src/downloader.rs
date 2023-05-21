use std::{fs::{File, self}, thread, time::{Duration, SystemTime}, io::Write};

use indicatif::{ProgressBar, ProgressStyle, ProgressState};
use reqwest::StatusCode;
use serde::Deserialize;
use size_format::SizeFormatterBinary;

use crate::{config::Config, log::Log, discord::{Discord, DiscordStatus}, time::format_millis};

#[derive(Deserialize)]
pub struct SearchResult {
  id: u32,
  artist: String,
  creator: String,
  title: String,
}

#[derive(Deserialize)]
pub struct MirrorError {
  error: String,
}

pub struct Downloader {
  offset: u32,
  cache: Vec<String>,
  discord: Discord,
  status: DiscordStatus,
  completed_file: File,
  sizes_file: File,
}

impl Downloader {
  pub fn new(cache: Vec<String>, discord: Discord) -> Self {
    Self {
      offset: 0,
      cache,
      discord,
      status: DiscordStatus {
        count: 0,
        size: 0,
        start: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
      },
      completed_file: File::options().write(true).open("./.data/completed").unwrap(),
      sizes_file: File::options().write(true).open("./.data/size").unwrap(),
    }
  }

  pub async fn crawl(&mut self, config: &Config) -> bool {
    let url = format!("https://catboy.best/api/v2/search?{}", config.get_querystring(self.offset));
    println!("{}", url);
    let search = match reqwest::get(url)
      .await.unwrap()
      .error_for_status() {
        Ok(res) => res.json::<Vec<SearchResult>>().await.unwrap(),
        Err(err) => {
          if let Some(status) = err.status() {            
            if status == StatusCode::INTERNAL_SERVER_ERROR {
              Log::error("Something is wrong in your query, please report this to the developer")
            } else {
              Log::error(format!("catboy.best seems to not have given you a proper response, please report this to the developer. ({})", status.as_u16()).as_str())
            }
          } else {
            Log::error("Something went HORRIBLY wrong.");
          }
          panic!()
        }
      };

    Log::success(format!("Loaded {} maps", search.len()).as_str());

    if search.len() == 0 {
      return false
    }

    self.offset += search.len() as u32;

    for result in search {
      if self.cache.contains(&result.id.to_string()) {
        Log::info(format!("Skipping {} - cached", result.id).as_str());
        continue
      }
      Log::info(format!("Downloading {} | {} - {} by {}", result.id, result.artist, result.title, result.creator).as_str());
      match self.download(config, &result).await {
        Ok(size) => {
          self.status.size += size;
        },
        Err(ratelimit) => {
          if ratelimit {
            Log::warn("Mirror reached ratelimit, pausing");
            thread::sleep(Duration::from_secs(6000));
          } else {
            Log::error("Something went wrong");
            panic!();
          }
        }
      }
      self.discord.update(&self.status);
      thread::sleep(Duration::from_secs(5));
    }

    true
  }
  
  async fn download(&mut self, config: &Config, map: &SearchResult) -> Result<u64, bool> {
    let start = SystemTime::now();

    let url = format!("https://catboy.best/d/{}{}", map.id, if !config.video { "n" } else { "" });
    let mut source = match reqwest::get(url).await {
      Ok(source) => source,
      Err(err) => {
        println!("{} {} {} {}", err.is_connect(), err.is_redirect(), err.is_request(), err.is_timeout());
        if err.is_timeout() {
          Log::error("Request timed out. Fucking WHY????????????????");
        } else {
          Log::error("Something went wrong");
        }
        panic!("{}", err)
      },
    };
    
    if let Some(content_type) = source.headers().get("content-type") {
      if content_type.to_str().unwrap().starts_with("application/json") {
        let data = source.json::<MirrorError>().await.unwrap();
        return Err(data.error == "Ratelimit")
      }
    }

    let content_length = str::parse::<u64>(source.headers().get("content-length").unwrap().to_str().unwrap()).unwrap();

    let pb = ProgressBar::new(content_length);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

    let mut size: u64 = 0;

    let path = format!("./songs/{}.osz", map.id);
    let mut dest = File::create(path.clone()).unwrap();
    while let Some(chunk) = source.chunk().await.unwrap() {
      dest.write_all(&chunk).unwrap();
      let new = size + chunk.len() as u64;
      size = new;
      pb.set_position(new);
    }

    thread::sleep(Duration::from_millis(50));

    if content_length != size {
      pb.abandon_with_message(format!("Failed to download map {}", map.id));
      // Log::error(format!("Failed to download map {}", map.id).as_str());
      fs::remove_file(path).unwrap();
      return Ok(0)
    }

    let duration = SystemTime::now().duration_since(start).unwrap().as_millis();
    pb.finish_with_message(format!("Finished in {} ({}B)", format_millis(duration), SizeFormatterBinary::new(size)));
    // Log::success(format!("Finished in {} ({}B)", format_millis(duration), SizeFormatterBinary::new(size)).as_str());

    self.completed_file.write(format!("{},", map.id).as_bytes()).unwrap();
    self.sizes_file.write(format!("{},", size).as_bytes()).unwrap();

    self.status.count += 1;

    Ok(size)
  }
}