#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      create_test_image
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

use std::io::Cursor;
use image::{
  codecs::png::PngEncoder, ColorType, ImageEncoder, Rgb, RgbImage
};
use base64::{
  Engine,
  engine::general_purpose,
};

#[tauri::command]
fn create_test_image(width: u32, height: u32) -> String {
  let mut img = RgbImage::new(width, height);
  for y in 0..height {
    for x in 0..width {
      // 適当にピクセル値を作成
      let r = (x * 255 / width) as u8;
      let g = (y * 255 / height) as u8;
      let b = (((x + y) / 2) * 255 / ((width + height) / 2)) as u8;
      img.put_pixel(x, y, Rgb([r, g, b]));
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