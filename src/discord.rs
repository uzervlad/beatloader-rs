use discord_presence::Client;
use size_format::SizeFormatterBinary;

#[derive(Debug)]
pub struct DiscordStatus {
  pub count: u32,
  pub size: u64,
  pub start: u64,
}

pub struct Discord {
  pub client: Client
}

#[allow(unused_must_use)]
impl Discord {
  pub fn new() -> Self {
    Self {
      client: Client::new(1107445406759145492),
    }
  }

  pub fn init(&mut self) {
    self.client.start();
  }

  pub fn start(&mut self) {
    self.client.set_activity(|act| {
      act.details("Running Beatloader v0.727")
        .state("Starting up...")
        .instance(false)
    });
  }

  pub fn update(&mut self, status: &DiscordStatus) {
    self.client.set_activity(|act| {
      act.details("Downloading from catboy.best")
        .state(format!("{} beatmaps ({:.2}B)", status.count, SizeFormatterBinary::new(status.size)))
        .timestamps(|t| {
          t.start(status.start)
        })
        .instance(false)
        .assets(|a| {
          a.large_image("logo").large_text("beatloader-rs")
        })
    });
  }
}