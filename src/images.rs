use opencv::prelude::*;
use opencv::types::*;
use opencv::core::*;
use opencv::imgcodecs;

/// Produce a chessboard calibration pattern with the given dimentions. The dimentions refer
/// to the internal "corners" on the board where four corners meet.
pub fn chessboard(nx: i32, ny: i32) -> Mat {
    let mut inverted = Mat::default().unwrap();
    let square_size = 50;
    let image_width = square_size * (nx + 1);
    let image_height = square_size * (ny + 1);
    let mat = Mat::new_size_with_default(Size::new(image_width, image_height), CV_8UC3, Scalar::all(0.)).unwrap();
    let mut color = 0u8;
    
    if nx % 2 == 0 || ny % 2 == 1 {
        panic!("chessboard width must be odd, height even");
    }
    
    // starts black on top-left corner (then inverted)
    let mut i = 0;
    for _ in 0..nx+1 {
        let mut j = 0;
        for _ in 0..ny+1 {
            let mut square = Mat::roi(&mat, Rect::new(i, j, square_size, square_size)).unwrap();
            if color > 0 {
                square.set(Scalar::all(color as f64)).unwrap();
            }
            color = color ^ 0xffu8;
            j += square_size;
        }
        
        i += square_size;
    }
    bitwise_not(&mat, &mut inverted, &Mat::default().unwrap()).unwrap();
    inverted
}

/// Produce a chessboard pattern and encode in the given image format.
pub fn chessboard_image(nx: i32, ny: i32, format: &str) -> VectorOfu8 {
  let data = chessboard(nx, ny);
  encode_image(&data, format)
}

pub fn encode_image(data: &Mat, format: &str) -> VectorOfu8 {
  let mut encoded = VectorOfu8::new();
  imgcodecs::imencode(&format, &data, &mut encoded, &VectorOfi32::new()).unwrap();
  encoded
}

#[allow(dead_code)]
pub fn pixel_png(r: u8, g: u8, b: u8) -> VectorOfu8 {
    let mat = Mat::new_size_with_default(Size::new(1, 1), CV_8UC3, Scalar::new(r as f64, g as f64, b as f64, 255.)).unwrap();
    encode_image(&mat, ".png")
}
