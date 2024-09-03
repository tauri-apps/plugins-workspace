const COMMANDS: &[&str] = &["get_latest_notification_data", "get_token", "subscribe_to_topic", "register_listener"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();
}
