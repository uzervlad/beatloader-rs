use std::{fs::{File, self}, thread, time::{Duration, SystemTime}, io::Write};

use async_recursion::async_recursion;
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

pub struct Downloader<'a> {
  offset: u32,
  cache: Vec<String>,
  discord: &'a mut Discord,
  status: DiscordStatus,
  completed_file: File,
  sizes_file: File,
}

const RETRY_COUNT: u8 = 5;

impl<'a> Downloader<'a> {
  pub fn new(cache: Vec<String>, discord: &'a mut Discord) -> Self {
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
    let url = format!("https://{}/api/v2/search?{}", config.host, config.get_querystring(self.offset));
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

    let mut skipped = 0;

    for result in search {
      if self.cache.contains(&result.id.to_string()) {
        skipped += 1;
        continue
      }
      
      if skipped > 0 {
        Log::info(format!("Skipped {} maps - cached", skipped).as_str());
        skipped = 0;
      }

      Log::info(format!("Downloading {} | {} - {} by {}", result.id, result.artist, result.title, result.creator).as_str());
      match self.download(config, &result, 0).await {
        Ok(size) => {
          self.status.size += size;
        },
        Err(err) => {
          match err.as_str() {
            "Ratelimit" => {
              Log::warn("Mirror reached ratelimit, pausing");
              thread::sleep(Duration::from_secs(6000));
            },
            "Map not available for download" => {
              Log::warn("Map is unavailable. Skipping...");
            },
            "Skip" => {
              Log::warn("Retry count exceeded. Skipping...");
            },
            _ => {
              Log::error("Something went horribly wrong");
              panic!("{}", err)
            }
          }
        }
      }
      let status = self.status.clone();
      self.discord.update(status);
      thread::sleep(Duration::from_secs(5));
    }

    true
  }

  #[async_recursion]
  async fn download(&mut self, config: &Config, map: &SearchResult, tries: u8) -> Result<u64, String> {
    if tries >= RETRY_COUNT {
      return Err("Skip".to_owned())
    }

    let start = SystemTime::now();

    let url = format!("https://{}/d/{}{}", config.host, map.id, if !config.video { "n" } else { "" });
    match reqwest::get(url).await {
      Err(_) => {
        Log::error("Something went wrong. Trying again.");
        self.download(config, map, tries + 1).await
      },
      Ok(mut source) => {
        if let Some(content_type) = source.headers().get("content-type") {
          if content_type.to_str().unwrap().starts_with("application/json") {
            let data = source.json::<MirrorError>().await.unwrap();
            return Err(data.error)
          }
        }
    
        let content_length = str::parse::<u64>(source.headers().get("content-length").unwrap().to_str().unwrap()).unwrap();
    
        let pb = ProgressBar::new(content_length);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.green}] {bytes}/{total_bytes} ({eta})")
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
    
        if content_length != size {
          pb.finish_and_clear();
          Log::error(format!("Failed to download map {}", map.id).as_str());
          fs::remove_file(path).unwrap();
          return Ok(0)
        }
    
        let duration = SystemTime::now().duration_since(start).unwrap().as_millis();
        pb.finish_and_clear();
        Log::success(format!("Finished in {} ({}B)", format_millis(duration), SizeFormatterBinary::new(size)).as_str());
    
        self.completed_file.write(format!("{},", map.id).as_bytes()).unwrap();
        self.sizes_file.write(format!("{},", size).as_bytes()).unwrap();
    
        self.status.count += 1;
    
        Ok(size)
      }
    }
  }
}