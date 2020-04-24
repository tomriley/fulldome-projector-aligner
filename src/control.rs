
/// format should be "png", "jpg" etc
pub fn post_image(control_url: &str, image_bytes: &[u8], format: &str) {
  let url = format!("{}/show_image", control_url);
  let ctype = format!("image/{}", format);

  let client = reqwest::blocking::Client::new();
  let _res = client.post(&url)
      .body(image_bytes.to_vec())
      .header("Content-Type", &ctype)
      .send().expect("Failed to post image to projector");
}

/// Send a command an optional json to the remote control URL
pub fn send_command(url: &str, command: &str, json: Option<serde_json::Value>) {
  let url = format!("{}/{}", url, command);
  let mut req = reqwest::blocking::Client::new().post(&url).header("Content-Type", "application/json");
  req = match json {
      Some(json) => {
        let json_str = json.to_string();
        req.body(json_str)
      },
      None => req
  };
  req.send().expect("failed to send comand to projector");
}
