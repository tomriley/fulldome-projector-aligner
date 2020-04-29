
use opencv::prelude::*;
use opencv::types::*;
use opencv::core::*;
use opencv::imgcodecs;
use super::PhysicalCamera;
use super::camera_calibration::Calibration;
use log::{info, debug};
use serde_json::json;
use serde::Deserialize;
use std::fs;


#[derive(Deserialize, Debug)]
struct CameraLocation {
    pub position: Vec<f32>,
    pub direction: Vec<f32>,
    pub up: Vec<f32>,
    pub fov: f32
}


/// get camera position relative to single aruco marker
pub fn locate_aruco_marker(calibration: &Calibration, photo: &mut Mat, marker_size: f32) {
    let mut ids = VectorOfi32::new();
    let mut corners = VectorOfVectorOfPoint2f::new();
    let mut rejected = VectorOfVectorOfPoint2f::new();
    let dict = opencv::aruco::get_predefined_dictionary(opencv::aruco::PREDEFINED_DICTIONARY_NAME::DICT_6X6_250).unwrap();
    let params = opencv::aruco::DetectorParameters::create().unwrap();
    
    // aruco lib can undistort for us so work on the original image...
    opencv::aruco::detect_markers(
        photo,
        &dict,
        &mut corners,
        &mut ids,
        &params,
        &mut rejected,
        &calibration.camera_matrix,
        &calibration.distortion_coefficients).expect("problem with aruco::detect_markers");
    
    // draw onto image (for debugging purposes)
    opencv::aruco::draw_detected_markers(photo, &corners, &ids, opencv::core::Scalar::all(0.)).expect("draw markers failed");
    imgcodecs::imwrite("locator-detected-markers.jpg", photo, &VectorOfi32::new()).unwrap();
    
    if corners.len() == 0 {
        panic!("No markers detected. Stopping.");
    } else if corners.len() > 1 {
        panic!("Multiple markers detected. Stopping.");
    }
    //info!("Found 1 aruco marker, top-left corner at {}, {}\n", corners[0][0].x, corners[0][0].y);
    
    let mut rvecs = VectorOfPoint3d::new();
    let mut tvecs = VectorOfPoint3d::new();
    let mut obj_points = VectorOfPoint3d::new(); // corners points of square
    
    opencv::aruco::estimate_pose_single_markers(
        &corners,
        marker_size,
        &calibration.camera_matrix,
        &calibration.distortion_coefficients,
        &mut rvecs,
        &mut tvecs,
        &mut obj_points
    ).expect("estimate pose failed");

    // TODO - sometimes the X axis is flipped - need a way to detect this and retry/fail
    // or just flip the axis if that fixes everything

    //for (int i=0 ; i<ids.size() ; i++) {
    //    opencv::aruco::drawAxis(original, cameraMatrix, distCoeffs, rvecs[i], tvecs[i], 0.03);
    //}

 //   let rvecs = rvecs.iter().map(|pt| vec3(pt.x, pt.y, pt.z)).collect();
 //   let tvecs = tvecs.iter().map(|pt| vec3(pt.x, pt.y, pt.z)).collect();

    let mut rvec = rvecs.iter().nth(0).unwrap();
    let mut tvec = tvecs.iter().nth(0).unwrap();

    // copy and convert to opengl axis layout
    rvec.x = rvec.x;
    rvec.y = -rvec.y;
    rvec.z = -rvec.z;
    
    tvec.x = tvec.x;
    tvec.y = -tvec.y;
    tvec.z = -tvec.z;

    let rvec_mat = Matx31::from([rvec.x, rvec.y, rvec.z]);
    let mut rm = Mat::default().unwrap(); // rotation matrix
    let mut jacobian = Mat::default().unwrap();
    opencv::calib3d::rodrigues(&rvec_mat, &mut rm, &mut jacobian).unwrap();
    
    let rm = rm.into_typed::<f64>().unwrap();
    
    let mut modelview = glm::mat4(
        *rm.at::<f64>(0).unwrap() as f32, *rm.at::<f64>(3).unwrap() as f32, *rm.at::<f64>(6).unwrap() as f32, 0.0,
        *rm.at::<f64>(1).unwrap() as f32, *rm.at::<f64>(4).unwrap() as f32, *rm.at::<f64>(7).unwrap() as f32, 0.0,
        *rm.at::<f64>(2).unwrap() as f32, *rm.at::<f64>(5).unwrap() as f32, *rm.at::<f64>(8).unwrap() as f32, 0.0,
        tvec.x as f32, tvec.y as f32, tvec.z as f32, 1.
    );
    
    // the aruco library gives us the pose at a 180 degree rotation in the X axis as
    // compared with our chessboard pose estimation, so rotate here
    modelview = glm::ext::rotate(&modelview, glm::radians(180.), glm::vec3(1., 0., 0.));
    let translation = modelview.c3;

    // calculate up and lookat vectors using the inverse modelview rotation
    // this assumes that our marker is at 0, 0, 0 facing into the positive z axis
    let mut rotation = modelview.clone();
    rotation.c3 = glm::vec4(0., 0., 0., 1.); // remove translation
    let inv_rotation = glm::transpose(&rotation);
    
    let mut negative = -translation;
    negative.w = 1.;

    let position = inv_rotation * negative;
    let dir = inv_rotation * glm::vec4(0., 0., -1., 1.);
    let up = inv_rotation * glm::vec4(0., 1., 0., 1.);

    let json = location_json_string(&position.truncate(3), &dir.truncate(3), &up.truncate(3) , calibration.fov);
    println!("{}", json);
}

pub fn update_physical_camera_location(physical_camera: &mut PhysicalCamera, json_fname: &str) {
    
    let json_str = fs::read_to_string(json_fname).expect("camera location json file not found");
    let cl: CameraLocation = serde_json::from_str(json_str.as_str()).unwrap();

    physical_camera.position = glm::vec3(cl.position[0], cl.position[1], cl.position[2]);
    physical_camera.look_at = glm::vec3(cl.direction[0], cl.direction[1], cl.direction[2]);
    physical_camera.up_dir = glm::vec3(cl.up[0], cl.up[1], cl.up[2]);
}


fn location_json_string(position: &glm::Vec3, dir: &glm::Vec3, up: &glm::Vec3, fov: f32) -> String {
    let json = json!({
        "position": position.as_array(),
        "direction": dir.as_array(),
        "up": up.as_array(),
        "fov": fov
    });

    serde_json::to_string_pretty(&json).unwrap()
}