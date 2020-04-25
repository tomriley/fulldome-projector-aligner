
use super::PhysicalCamera;
use super::math::*;
use glm::*;
use glm::ext::*;
use std::f32::consts;


pub enum SurfaceType {
    HemisphericalDome {radius: f32},
    Wall
}

/// convert a point in camera photo space to a 3d point on the projection surface in scene space
pub fn camera_to_scene(surface_type: &SurfaceType, physical_camera: &PhysicalCamera, point: glm::Vec2, image_width: i32, image_height: i32) -> Result<glm::Vec3, &'static str> {
    match surface_type {
        SurfaceType::HemisphericalDome{radius} => camera_to_scene_dome(point, image_width, image_height, *radius),
        SurfaceType::Wall => camera_to_scene_wall(physical_camera, point, image_width, image_height)
    }
}

fn camera_to_scene_dome(corner_pt: glm::Vec2, image_width: i32, image_height: i32, dome_radius: f32) -> Result<glm::Vec3, &'static str> {
    // do what we would do in the dome with fisheye camera
    
    // figure out height of dome at corner_pt
    // For now we're assuming the image is square and filled 100% with the fisheye camera image
    let h = image_height as f32;
    let w = image_width as f32;

    let mut corner_pt = corner_pt.clone();
    
    // normalize corner point position in 2D space relative to the center of the image
    corner_pt.y -= h * 0.5;
    corner_pt.x -= w * 0.5;
    corner_pt.y /= h * 0.5;
    corner_pt.x /= w * 0.5;
    
    // distance to point from center of dome in image (fisheye) space
    let v = (corner_pt.x * corner_pt.x + corner_pt.y * corner_pt.y).sqrt();
    
    // v is proportional to distance around the dome surface which is proportional
    // to angle away from the Y axis. Use this to calculate normalized height of point.
    let angle1 = (consts::PI/2.)*(1.-v); // angle is radians away from the XZ plane
    let real_y = angle1.sin(); // use this to calculate height at point
    let real_v = angle1.cos(); // real distance of point from center of dome on XZ plane
    
    // angle of point around the Y axis
    let angle2 = atan2(corner_pt.x, corner_pt.y);
    
    // we use this to calculate the real X and Y coords
    let real_x = angle2.cos() * real_v;
    let real_z = angle2.sin() * real_v;
    
    //float test = realY*realY + realX*realX + realZ*realZ; // should equal 1
    
    if v > 1.0 {
        // point lies outside of dome
        Err("point passed to camera_to_scene_dome_fisheye seems to lie outside of the dome")
    } else {
        //NSLog(@"normalized dome height is %f", realY);
        Ok(vec3(real_x, real_y, real_z) * dome_radius)
    }
}

// wall surface specific. wall is assumed to be at z = 0
fn camera_to_scene_wall(camera: &PhysicalCamera, pt: glm::Vec2, image_width: i32, image_height: i32) -> Result<glm::Vec3, &'static str> {
    // unproject camera point into camera based scene
    // FIXME shouldn't be rebuilding these for each point
    // consider simpifying and inline this where it's needed
    // actually, maybe it's fine as this is only used for mapping the projected
    // points
    let model = look_at(camera.position, camera.look_at, camera.up_dir);
    let proj = perspective(radians(camera.calibration.fov), (image_width as f32) / (image_height as f32), 0.1_f32, 1000_f32);
    
    let pt = un_project(vec3(pt.x, image_height as f32 - pt.y, 1.), // 1 means at the back of the depth range
                        &model,
                        &proj,
                        vec4(0., 0., image_width as f32, image_height as f32))?;
    
    // now intersect with the XY plain
    let zmag = (pt.z - camera.position.z).abs();
    let scene_pt = ((pt - camera.position) / zmag) * (0. /*wall z*/ - camera.position.z).abs() + camera.position;
    
    Ok(scene_pt)
}
