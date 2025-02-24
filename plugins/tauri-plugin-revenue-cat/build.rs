const COMMANDS: &[&str] = &["ping", "createCustomer", "fetchProjectOverviewData", "getCustomersData"];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .android_path("android")
    .ios_path("ios")
    .build();
}
