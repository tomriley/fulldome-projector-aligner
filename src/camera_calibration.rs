
use opencv::prelude::*;
use opencv::core::*;
use xmltree::Element;
use std::fs::File;
use glm::*;
use log::{info};

pub struct Calibration {
    pub camera_matrix: Matx33d,
    pub distortion_coefficients: Mat,
    pub fov: f32
}

pub fn load_calibration_file(fname: &str) -> Option<Calibration> {
    // Load the camera calibration from file
    let mut root_elm = Element::parse(File::open(fname).unwrap()).unwrap();
    let node = root_elm.get_mut_child("camera_matrix").expect("can't find camera_matrix element");
    let data = node.get_mut_child("data").expect("can't find camera_matrix data");
    let values = data.get_text().unwrap();
    let mut floats = [0_f64; 9];
    for (i, word) in values.split_ascii_whitespace().enumerate() {
        floats[i] = word.parse().unwrap();
    }
    let camera_matrix = Matx33d::from(floats);

    let node = root_elm.get_mut_child("distortion_coefficients").expect("can't find distortion_coefficients element");
    let data = node.get_mut_child("data").expect("can't find distortion_coefficients data");
    let values = data.get_text().unwrap();
    let floats: Vec<f64> = values.split_ascii_whitespace().map(|word| word.parse().unwrap()).collect();
    let distortion_coefficients = Mat::from_slice(&floats).unwrap();

    let node = root_elm.get_mut_child("image_height").expect("can't find image_height element");
    let img_height: i32 = node.get_text().as_deref().unwrap().parse().unwrap();

    // get the camera FOV from the intrinsic camera matrix
    let fy: f32 = camera_matrix.get((1, 1)).unwrap().clone() as f32;
    let fov = glm::degrees(2. * atan2(img_height as f32, 2. * fy));

    info!("camera matrix and distortion coefficients loaded from {}", &fname);
    info!("physical camera field of view calculated as {} degrees", fov);

    Some(Calibration {camera_matrix: camera_matrix, distortion_coefficients: distortion_coefficients, fov: fov})
}
