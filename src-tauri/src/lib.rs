mod math;

use math::{calculate_partial_sums, find_analytical_crossings, get_zeta_value, calculate_zeta_path};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_log::Builder::default().build())
    .invoke_handler(tauri::generate_handler![
        calculate_partial_sums,
        find_analytical_crossings,
        get_zeta_value,
        calculate_zeta_path
    ])
    .setup(|_app| {
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
