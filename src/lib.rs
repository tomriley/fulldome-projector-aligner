
use opencv::prelude::*;
use opencv::types::*;
use opencv::core::*;
use opencv::imgcodecs;
use opencv::highgui;
use opencv::imgproc::*;
use opencv::calib3d::*;
use glm::*;
use glm::ext::*;
use serde_json::json;
use std::io::prelude::*;
use log::{info, warn, debug, error};
use regex::Regex;
use lazy_static::*;

mod math;
mod photo;
mod images;
mod control;
mod locator;
mod surfaces;
mod camera_calibration;

use camera_calibration::Calibration;

pub struct PhysicalCamera {
    pub position: glm::Vec3,
    pub look_at: glm::Vec3,
    pub up_dir: glm::Vec3
}

struct VirtualCamera {
    pub position: glm::Vec3,
    pub look_at: Option<glm::Vec3>, // this is calculated during calibration
    pub up_dir: glm::Vec3,
    pub fov: Option<f32> // this is calculated during calibration
}

struct Projector {
    resolution_x: i32,
    resolution_y: i32,
}

impl Projector {
    /// aspect ratio of projector output as a fraction (width/height)
    fn aspect_ratio(&self) -> f32 {
        self.resolution_x as f32 / self.resolution_y as f32
    } 
}

#[derive(Clone, Copy)]
pub struct WarpResolution {
    width: i32,
    height: i32
}

impl WarpResolution {
    pub fn parse(input: &str) -> Result<WarpResolution, String> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?P<width>\d+)x(?P<height>\d+)").unwrap();
        }
        let caps = RE.captures(input).expect("Failed to parse warp resolution input string");
        Ok(WarpResolution {
            width: caps["width"].parse().unwrap(),
            height: caps["height"].parse().unwrap()
        })
    }
}

pub fn warp_hemispherical_dome(radius: f32, camera_cal_fname: &str, control_url: Option<&str>, photo_url: Option<&str>, warp_res: WarpResolution, _verbosity: i32) {
    info!("building warp for hemispherical dome...");
    let calibration = camera_calibration::load_calibration_file(camera_cal_fname).expect("load of calibration XML failed");
    let surface = surfaces::HemisphericalDome{radius: radius};
    let physical_camera = PhysicalCamera {    
        // camera position (should be suppied by user)
        position: vec3(0., 0., 0.),
        look_at: vec3(0., 1., 0.),
        up_dir: vec3(0., 0., 1.)
    };
    let projector = Projector {
        resolution_x: 1920,
        resolution_y: 1080
    };
    gen_warp(surface, physical_camera, projector, calibration, control_url, photo_url, warp_res).unwrap();
}

pub fn warp_wall(camera_cal_fname: &str, control_url: Option<&str>, photo_url: Option<&str>, warp_res: WarpResolution, _verbosity: i32) {
    info!("builing warp for flat 3D wall...");
    let calibration = camera_calibration::load_calibration_file(camera_cal_fname).expect("load of calibration XML failed");
    let surface = surfaces::Wall{};
    let physical_camera = PhysicalCamera {    
        // camera position (should be suppied by user)
        position: vec3(0., 0., 2.),
        look_at: vec3(0., 0., 0.),
        up_dir: vec3(0., 0., 1.)
    };
    let projector = Projector {
        resolution_x: 1920,
        resolution_y: 1080
    };
    gen_warp(surface, physical_camera, projector, calibration, control_url, photo_url, warp_res).unwrap();
}

fn gen_warp<G>(geometry: G, physical_camera: PhysicalCamera, projector: Projector, calibration: Calibration, control_url: Option<&str>, _photo_url: Option<&str>, warp_res: WarpResolution) -> opencv::Result<String>
    where G: surfaces::CameraToScene {

    let mut virtual_camera = VirtualCamera {
        position: vec3(0., 0., 0.),
        look_at: None,
        up_dir: vec3(0., 1., 0.),
        fov: None,
    };
    
    let chessboard = images::chessboard_image(warp_res.width, warp_res.height, ".png");
    
    // show chessboard image on first projector
    match &control_url {
        Some(url) => control::post_image(&url, &chessboard.to_slice(), "png"),
        None => {
            info!("Please display the full-screen chessboard pattern on the projector and press any key");
            std::io::stdin().bytes().next();
            info!("Continuing...");
        }
    }

    // take photo
    let photo = fetch_simulator_photo();
    // take photo
    //let img = imgcodecs::imread("test.jpg", imgcodecs::IMREAD_COLOR)?;
    
    // convert to greyscale and invert back to expected color layout and white border
    // required for the opencv corner detection to work
    let mut gray = Mat::default()?;
    let mut inverted_img = Mat::default()?;
    //let mut camera_v_fov: f32 = 0.;

    cvt_color(&photo, &mut gray, COLOR_BGR2GRAY, 1)?;
    bitwise_not(&gray, &mut inverted_img, &Mat::default()?)?;
    
    imgcodecs::imwrite("alignment-inverted.jpg", &inverted_img, &VectorOfi32::new())?;
    
    let mut undistorted_img = Mat::default()?;
    undistort(&inverted_img, &mut undistorted_img, &calibration.camera_matrix, &calibration.distortion_coefficients, &calibration.camera_matrix)?;
    inverted_img = undistorted_img;
    
    imgcodecs::imwrite("alignment-undistorted.jpg", &inverted_img, &VectorOfi32::new())?;
    
    // find chessboard corners
    let mut point_buffer = VectorOfPoint2f::new();
    let board_size = Size::new(warp_res.width, warp_res.height);
    let found = find_chessboard_corners(&inverted_img, board_size, &mut point_buffer, CALIB_CB_ADAPTIVE_THRESH).unwrap();
    
    // draw found chessboard corners to image file
    if true {
        let mut color_img = Mat::default()?;
        cvt_color(&gray, &mut color_img, COLOR_GRAY2BGR, 3)?;
        draw_chessboard_corners(&mut color_img, board_size, &point_buffer, found)?;
        imgcodecs::imwrite("alignment-corners.jpg", &color_img, &VectorOfi32::new())?;
    }

    if !found {
        error!("No chessboard corners detected");
    }

    // corner subpix analysis
    corner_sub_pix(&inverted_img, &mut point_buffer, board_size, Size::new(-1, -1),
                     TermCriteria::new(3, 30, 0.1f64).unwrap())?; // 3 = COUNT + EPS
    
    // convert to vector of glm::Vec2
    let point_buffer: Vec<glm::Vec2> = point_buffer.iter().map(|pt| vec2(pt.x, pt.y)).collect();

    // find center of projection area (in camera image space)
    // FIXME assuming center point relative to physical camera is same as center of projection
    // frustum is probably wrong. May be especially true for fisheye camera images. Should we convert
    // each point with camera_to_scene, 
    // calculate look at direction and required simulator yfov
    let mut avg = vec2(0., 0.);
    for p in point_buffer.iter() { avg = avg + *p }
    avg = avg / point_buffer.len() as f32;
    
    debug!("Projection area center point is {:?}", avg);

    // central point of projection surface in scene space
    let look_at_center = geometry.camera_to_scene(&physical_camera, &calibration, avg, photo.cols(), photo.rows()).expect("camera to scene failed for average center point??");
    virtual_camera.look_at = Some(look_at_center);
    
    let trans = look_at(virtual_camera.position, virtual_camera.look_at.unwrap(), virtual_camera.up_dir);
    
    let mut max_rad = -1_f32;
    for point in point_buffer.iter() {
        let scene = geometry.camera_to_scene(&physical_camera, &calibration, *point, photo.cols(), photo.rows()).unwrap();
        let scene = vec4(scene.x, scene.y, scene.z, 1.);
        let eye_relative = trans * scene;
        
        let rad = atan(eye_relative.y.abs() / eye_relative.z.abs());
        if rad > max_rad { max_rad = rad; }
    }
    
    virtual_camera.fov = Some(glm::degrees(max_rad) * 2.1);
    
    info!("eyePoint = {:?} lookAt = {:?} fovY = {:?}", virtual_camera.position, virtual_camera.look_at, virtual_camera.fov.unwrap());
    
    let mut uv_coords = vec![];

    for point in point_buffer.iter() {
        // Convert point in camera space to a point in 3d world space
        let scene_coord = geometry.camera_to_scene(&physical_camera, &calibration, *point, photo.cols(), photo.rows()).unwrap();

        // project this scene coord into our viewer based pre-rendered viewport, returns normalized screen coord
        // to support a dynamic eye point (e.g. head tracking), it's from this point that we would need to do
        // calculations realtime within the render pipeline to shift the warp around. this would create a VR effect.
        let target_screen_point = project_scene_point(scene_coord, &virtual_camera, projector.aspect_ratio()).unwrap();

        // We now have the coord pixel of the render buffer that should be warped to the current chessboard corner
        uv_coords.push(target_screen_point);
    }

    // Build final "calibration" JSON document
    let warp: Vec<&[f32; 2]> = uv_coords.iter().map(|p| p.as_array()).collect();

    let json = json!({
        "fov": virtual_camera.fov,
        "eye": virtual_camera.position.as_array(),
        "lookAt": virtual_camera.look_at.unwrap().as_array(),
        "up": virtual_camera.up_dir.as_array(),
        "warpResX": warp_res.width,
        "warpResY": warp_res.height,
        "warp": warp
    });

    // show chessboard image on first projector
    match &control_url {
        Some(url) => control::send_command(&url, "set_calibration", Some(json)),
        None => {
            info!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
    }
    
    Ok(String::from("calibration json..."))
}

/// Given virtual camera details, calculate normalized screen position of the point in 3D space
fn project_scene_point(scene_pos: glm::Vec3, virtual_camera: &VirtualCamera, projector_aspect_ratio: f32) -> Result<glm::Vec2, &'static str> {
    let model = glm::ext::look_at(virtual_camera.position, virtual_camera.look_at.unwrap(), virtual_camera.up_dir);
    let proj = glm::ext::perspective(glm::radians(virtual_camera.fov.expect("scene camera fov should have been calculated")), projector_aspect_ratio, 0.1, 100.);
    
    let screen_pos = math::project(vec3(scene_pos.x, scene_pos.y, scene_pos.z), &model, &proj, vec4(0., 0., 1., 1.))?;
    if screen_pos.x < 0. || screen_pos.y < 0. || screen_pos.x > 1. || screen_pos.y > 1. {
        warn!("a point in the scene space projected off screen (in project_scene_point)");
        //screen_pos.x = screen_pos.x.max(0.);
        //screen_pos.y = screen_pos.y.max(0.);
    }
    
    Ok(screen_pos.truncate(2))
}


fn fetch_simulator_photo() -> Mat {
    let url = "http://localhost:8080/photo/png";
    let client = reqwest::blocking::Client::new();
    let res = client.get(url).send().expect("Failed to fetch simulator photo");
    let bytes = res.bytes().expect("response didn't contain any data?");
    
    let data = Mat::from_slice(&bytes[..]).unwrap();
    imgcodecs::imdecode(&data, imgcodecs::IMREAD_COLOR).unwrap()
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