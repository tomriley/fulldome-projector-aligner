
use opencv::prelude::*;
use opencv::highgui;
use log::{warn, info, error, debug};
use std::fs::{File};
use std::io::{Read, ErrorKind};
use tempfile::NamedTempFile;
use std::{thread::sleep, process::{exit, Command}};

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

    // take the second of two photos because of issues with sony cameras
    let file = NamedTempFile::new().unwrap();
    let fpath = file.path().to_str().unwrap();
    for _ in 0..2 {
        let result = Command::new("gphoto2")
                .args(&["--capture-image-and-download", "--force-overwrite", "--filename", fpath])
                .output();
        match result {
            Ok(output) => {
                debug!("gphoto2 stdout: {}", String::from_utf8(output.stdout).unwrap());
                debug!("gphoto2 stderr: {}", String::from_utf8(output.stderr).unwrap());
            }
            Err(err) => {
                match err.kind() {
                    ErrorKind::NotFound => {
                        error!("gphoto2 executable not found. Is the gphoto2 package not installed?");
                        exit(-1);
                    },
                    _ => {
                        error!("Error while running gphoto2: {},", err.to_string());
                        exit(-1);
                    }
                };
            }
        }
        
        sleep(std::time::Duration::from_millis(1000));
    }
        
    let mut buffer = Vec::new();
    let mut file = File::open(fpath).unwrap();
    file.read_to_end(&mut buffer).unwrap();
    info!("returning gphoto image data");
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