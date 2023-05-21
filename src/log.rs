use colored::Colorize;

pub struct Log;

impl Log {
  #[allow(dead_code)]
  pub fn print(s: &str) {
    println!("{}", s)
  }

  pub fn error(s: &str) {
    println!("{}", s.red().bold())
  }

  pub fn warn(s: &str) {
    println!("{}", s.bright_yellow())
  }

  pub fn success(s: &str) {
    println!("{}", s.bright_green())
  }

  pub fn info(s: &str) {
    println!("{}", s.bright_cyan())
  }
}