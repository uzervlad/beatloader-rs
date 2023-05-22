use std::{sync::{mpsc::{self, Sender}, Arc, Mutex}, thread, time::Duration};

use discord_rpc_client::Client;
use size_format::SizeFormatterBinary;

#[derive(Debug, Clone)]
pub struct DiscordStatus {
  pub count: u32,
  pub size: u64,
  pub start: u64,
}

pub struct Discord {
  running: Arc<Mutex<bool>>,
  sender: Sender<DiscordStatus>,
}

#[allow(unused_must_use)]
impl Discord {
  pub fn new() -> Self {
    let running = Arc::new(Mutex::new(false));
    let (sender, receiver) = mpsc::channel::<DiscordStatus>();

    let mut client = Client::new(1107445406759145492);
    client.start();

    let running_clone = Arc::clone(&running);
    thread::spawn(move || {
      client.set_activity(|act| {
        act.details("Running Beatloader v0.727")
          .state("Starting up...")
          .instance(false)
          .assets(|a| {
            a.large_image("logo").large_text("beatloader-rs")
          })
      });

      let mut run = true;

      while run {
        let status = receiver.recv_timeout(Duration::from_secs(2));
        if let Ok(status) = status {
          client.set_activity(|act| {
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

        run = running_clone.lock().unwrap().clone();
      }
    });

    Self {
      running, sender
    }
  }

  pub fn update(&mut self, status: DiscordStatus) {
    self.sender.send(status);
  }

  pub fn stop(&mut self) {
    let mut r = self.running.lock().unwrap();
    *r = false;
  }
}