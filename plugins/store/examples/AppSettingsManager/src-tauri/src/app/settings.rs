use tauri_plugin_store::Store;

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub launch_at_login: bool,
    pub theme: String,
}

impl AppSettings {
    pub fn load_from_store<R: tauri::Runtime>(
        store: &Store<R>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let launch_at_login = store
            .get("appSettings.launchAtLogin")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let theme = store
            .get("appSettings.theme")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "dark".to_string());

        Ok(AppSettings {
            launch_at_login,
            theme,
        })
    }
}
