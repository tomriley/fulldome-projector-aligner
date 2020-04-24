
use opencv::prelude::*;
use opencv::imgcodecs;
use log::{warn, info};

pub trait PhotoProvider {
    fn take_photo(&self) -> Mat;
}

/// Real camera accessed via USB (with the gphoto2 library).
pub struct TetheredCamera {}

/// Performs a simple GET request to an image URL.
pub struct RemoteHttpCamera {
    pub url: String
}

/// Mock camera that simply returns contents of an existing image file.
pub struct SingleImageFile {
    pub image_file_name: String
}

impl PhotoProvider for SingleImageFile {
    /// Just returns the specified image file every time
    fn take_photo(&self) -> Mat {
        warn!("{} is being provided as a camera photo", &self.image_file_name);
        imgcodecs::imread(&self.image_file_name, imgcodecs::IMREAD_COLOR).expect("problem reading image file")
    }
}

impl PhotoProvider for RemoteHttpCamera {
    /// Fetch photo with a simple GET request to the URL
    fn take_photo(&self) -> Mat {
        info!("fetching camera photo from {}", &self.url);
        let client = reqwest::blocking::Client::new();
        let res = client.get(&self.url).send().expect("failed to request from remote camera URL");
        let bytes = res.bytes().expect("response didn't contain image data");
        Mat::from_slice(&bytes[..]).unwrap()
    }
}

