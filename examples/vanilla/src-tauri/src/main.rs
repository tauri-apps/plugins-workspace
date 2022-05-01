#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_single_instance::init(
      |argv, cwd, close_new_instance| {
        println!("{argv:?}, {cwd}");
        close_new_instance();
      },
    ))
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
