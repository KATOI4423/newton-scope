/// フラクタル計算処理を行うライブラリ
/// 

use image::{
    codecs::png::PngEncoder, ColorType, ImageEncoder, Rgb, RgbImage
};
use base64::{
    Engine,
    engine::general_purpose,
};
use once_cell::sync::Lazy;
use std::{
    i32, io::Cursor, sync::{
        Arc,
        RwLock,
    }
};
use num_complex::{Complex, ComplexFloat};
use formulac::{compile, variable::{UserDefinedTable, Variables}};

static VARIABLES: Lazy<formulac::Variables> = Lazy::new(|| Variables::new());
static USERTABLE: Lazy<formulac::UserDefinedTable> = Lazy::new(|| UserDefinedTable::new());

static FUNC_OBJ: Lazy<RwLock<Arc<dyn Fn(&[Complex<f64>]) -> Complex<f64> + Send + Sync + 'static>>> = Lazy::new(|| {
    let expr = compile("z^3 - 1", &["z"], &VARIABLES, &USERTABLE).unwrap();
    RwLock::new(Arc::new(expr))
});

static DIFF_OBJ: Lazy<RwLock<Arc<dyn Fn(&[Complex<f64>]) -> Complex<f64> + Send + Sync + 'static>>> = Lazy::new(|| {
    let expr = compile("3*z^2", &["z"], &VARIABLES, &USERTABLE).unwrap();
    RwLock::new(Arc::new(expr))
});

#[tauri::command]
pub fn update_formula(new_formula: String) {
    let expr = compile(
        &new_formula, &["z"], &VARIABLES, &USERTABLE
    ).unwrap();
    let mut w = FUNC_OBJ.write().unwrap();
    *w = Arc::new(expr);

    let expr = compile(
        &format!("diff({}, z)", new_formula),
        &["z"], &VARIABLES, &USERTABLE
    ).unwrap();
    let mut w = DIFF_OBJ.write().unwrap();
    *w = Arc::new(expr);
}

fn exec_newton_method(z: &Complex<f64>, a: &Complex<f64>) -> Complex<f64> {
    let f = FUNC_OBJ.read().unwrap();
    let df = DIFF_OBJ.read().unwrap();
    
    z - a * f(&[*z])/df(&[*z])
}

fn jet_from_i32(value: i32, max:i32) -> Rgb<u8> {
    let t = (value as f64 / max as f64).clamp(0.0, 1.0);
    let rgb: [u8; 3] = [3.0, 2.0, 1.0].map(
        |n|
            ((1.5 - (4.0 * t - n).abs()).clamp(0.0, 1.0) * 255.0) as u8
    );

    Rgb(rgb)
}

fn calc_pixel_value(x: u32, max_x: u32, y: u32, max_y: u32) -> Rgb<u8> {
    let max = 256;
    let calc_coor = |x: u32, max: u32| -> f64 {
        (x as f64) / (max as f64) * 4.0 - 2.0
    };

    let mut z = Complex::new(
        calc_coor(x, max_x), calc_coor(y, max_y)
    );
    let mut z_pre = z;
    let a = Complex::ONE;
    let mut cnt = 0;

    loop {
        z = exec_newton_method(&z_pre, &a);
        if z.is_nan() || ((z - z_pre).abs() < 1.0e-12) {
            break;
        }

        cnt += 1;
        if cnt == max {
            break;
        }

        z_pre = z;
    }

    jet_from_i32(cnt, max)
}

#[tauri::command]
pub fn create_fractal_image(width: u32, height: u32) -> String {
  let mut img = RgbImage::new(width, height);
  for y in 0..height {
    for x in 0..width {
      img.put_pixel(x, y, calc_pixel_value(x, width, y, height));
    }
  }

  // PNGにエンコードしてBase64化
  let mut buf = Cursor::new(Vec::new());
  let encoder = PngEncoder::new(&mut buf);
  encoder
    .write_image(&img, img.width(), img.height(), ColorType::Rgb8.into())
    .unwrap();
  general_purpose::STANDARD.encode(buf.into_inner())
}
