use reqwest::blocking::Response;

/// format should be "png", "jpg" etc
pub fn post_image(
    control_url: &str,
    image_bytes: &[u8],
    format: &str,
) -> reqwest::Result<Response> {
    let url = format!("{}/show_image", control_url);
    let ctype = format!("image/{}", format);
    let client = reqwest::blocking::Client::new();
    client
        .post(&url)
        .body(image_bytes.to_vec())
        .header("Content-Type", &ctype)
        .send() //.expect("Failed to post image to projector");
}

/// Send a command and optional json body to the remote control URL
pub fn send_command(url: &str, command: &str, json_str: &str) {
    let url = format!("{}/{}", url, command);
    let req = reqwest::blocking::Client::new()
        .post(&url)
        .header("Content-Type", "application/json")
        .body(String::from(json_str));
    req.send().expect("failed to send comand to projector");
}
