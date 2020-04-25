
use opencv::prelude::*;
use opencv::imgcodecs;
use opencv::highgui;
use log::{warn, info};

pub enum CameraType {
    TetheredCamera,
    RemoteHttpCamera{url: String},
    SingleImageFile{path: String}
}

/// Acquire a photo
pub fn capture_photo(camera_type: CameraType) -> Mat {
    match camera_type {
        CameraType::TetheredCamera => panic!("not implemented"),
        CameraType::RemoteHttpCamera{url} => fetch_photo_from_url(&url),
        CameraType::SingleImageFile{path} => load_from_file(&path)
    }
}

/// Just returns the specified image file every time
fn load_from_file(path: &str) -> Mat {
    warn!("{} is being provided as a camera photo", &path);
    imgcodecs::imread(&path, imgcodecs::IMREAD_COLOR).expect("problem reading image file")
}

fn fetch_photo_from_url(url: &str) -> Mat {
    info!("fetching camera photo from {}", &url);
    let client = reqwest::blocking::Client::new();
    let res = client.get(url).send().expect("failed to request from remote camera URL");
    let bytes = res.bytes().expect("response didn't contain image data");
    Mat::from_slice(&bytes[..]).unwrap()
}

// Show encoded image contained in mat and wait
#[allow(dead_code)]
fn imdebug(image: &Mat) -> opencv::Result<()> {
    let wname = "photo";
    highgui::named_window(wname, 1)?;
    highgui::imshow(wname, &image)?;

    loop {
        if highgui::wait_key(10)? > 0 {
            break;
        }
    }

    Ok(())
}