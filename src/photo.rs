
use opencv::prelude::*;
use opencv::highgui;
use log::{warn, info};
use std::fs::{File};
use std::io::{Read};
use std::process::Command;

pub enum CameraType {
    TetheredCamera,
    RemoteHttpCamera{url: String},
    SingleImageFile{path: String}
}

/// Acquire a photo
pub fn capture_photo(camera_type: CameraType) -> Mat {
    match camera_type {
        CameraType::TetheredCamera => take_photo(),
        CameraType::RemoteHttpCamera{url} => fetch_photo_from_url(&url),
        CameraType::SingleImageFile{path} => load_from_file(&path)
    }
}

/// Just returns the specified image file every time
fn load_from_file(path: &str) -> Mat {
    warn!("{} is being provided as a camera photo", &path);
    let mut buffer = Vec::new();
    let mut file = File::open(path).unwrap();
    file.read_to_end(&mut buffer).unwrap();
    Mat::from_slice(buffer.as_slice()).unwrap()
}

fn fetch_photo_from_url(url: &str) -> Mat {
    info!("fetching camera photo from {}", &url);
    let client = reqwest::blocking::Client::new();
    let res = client.get(url).send().expect("failed to request from remote camera URL");
    let bytes = res.bytes().expect("response didn't contain image data");
    Mat::from_slice(&bytes[..]).unwrap()
}

fn take_photo() -> Mat {
    // Capturing and downloading the same photo isn't reliable on either the Sony
    // a5100 or a6000. From looking at the gphoto2 command line app/shell it seems that
    // some kind of USB event comes from the camera to indicate that the new data is available for
    // download. For now, I'm just going to use the command line app to capture, wait for,
    // and downoad the image.
    /*let mut context = gphoto::Context::new().unwrap();
    let mut camera = gphoto::Camera::autodetect(&mut context).unwrap();
    let capture = camera.capture_image(&mut context).unwrap();
    let mut file = gphoto::FileMedia::create(Path::new(&*capture.basename())).unwrap();
    camera.download(&mut context, &capture, &mut file).unwrap();
    */

    let output = Command::new("gphoto2")
            .args(&["--capture-image-and-download", "--force-overwrite", "--filename", "my-photo.jpg"])
            .output()
            .expect("failed to execute process");

    info!("gphoto2 stdout: {}", String::from_utf8(output.stdout).unwrap());
    info!("gphoto2 stderr: {}", String::from_utf8(output.stderr).unwrap());
    
    let mut buffer = Vec::new();
    let mut file = File::open("my-photo.jpg").unwrap();
    file.read_to_end(&mut buffer).unwrap();
    info!("returning image data");
    Mat::from_slice(buffer.as_slice()).unwrap()
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