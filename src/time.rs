pub fn format_millis(millis: u128) -> String {
  if millis >= 1000 {
    format!("{:.2}s", millis as f64 / 1000.)
  } else {
    format!("{}ms", millis)
  }
}