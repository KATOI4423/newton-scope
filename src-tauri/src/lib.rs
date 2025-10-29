mod calculate;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  calculate::initialize();

  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      calculate::set_formula,
      calculate::initialize,
      calculate::get_default_formula,
      calculate::get_center_str,
      calculate::get_scale_str,
      calculate::get_size,
      calculate::get_max_iter,
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
    .plugin(tauri_plugin_dialog::init())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
