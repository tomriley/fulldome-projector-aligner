
use glm::*;

pub fn project<T: BaseFloat>(object: Vector3<T>, model: &Matrix4<T>, proj: &Matrix4<T>, viewport: Vector4<T>) -> Result<Vector3<T>, &'static str> {
  let mut tmp = object.extend(T::one());
  tmp = *model * tmp;
  tmp = *proj * tmp;

  tmp = tmp / tmp.w;
  //#		if GLM_DEPTH_CLIP_SPACE == GLM_DEPTH_ZERO_TO_ONE
  //			tmp.x = tmp.x * static_cast<T>(0.5) + static_cast<T>(0.5);
  //			tmp.y = tmp.y * static_cast<T>(0.5) + static_cast<T>(0.5);
  //#		else
  tmp = tmp * T::from(0.5).unwrap() + T::from(0.5).unwrap();
  //#		endif
  tmp[0] = tmp[0] * viewport[2] + viewport[0];
  tmp[1] = tmp[1] * viewport[3] + viewport[1];

  Ok(tmp.truncate(3))
}


// https://www.khronos.org/opengl/wiki/GluProject_and_gluUnProject_code
// todo return Result
pub fn un_project<T: BaseFloat>(window: Vector3<T>, model: &Matrix4<T>, proj: &Matrix4<T>, viewport: Vector4<T>) -> Result<Vector3<T>, &'static str> {
let inverse = proj.mul_m(model).inverse().unwrap();
let mut tmp = window.extend(T::one());
tmp.x = (tmp.x - viewport.x) / viewport.z;
  tmp.y = (tmp.y - viewport.y) / viewport.w;
//		if GLM_DEPTH_CLIP_SPACE == GLM_DEPTH_ZERO_TO_ONE
//			tmp.x = tmp.x * static_cast<T>(2) - static_cast<T>(1);
//			tmp.y = tmp.y * static_cast<T>(2) - static_cast<T>(1);
//		else
tmp = (tmp * T::from(2).unwrap()) - T::one();
//		endif

  let mut obj = inverse * tmp; //.mul_v(tmp);
  
  if obj.w == T::zero() {
      return Err("un_project failed, point not within screen bounds?");
  }

obj = obj / obj.w;

  Ok(obj.truncate(3))
}