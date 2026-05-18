#[cfg(test)]
#[test]
fn test_config_deserialization() {
    let config_json = serde_json::json!({
        "clearNotificationsOnAppFocus": true
    });
    let config: crate::models::PluginConfig = serde_json::from_value(config_json).unwrap();
    assert!(config.clear_notifications_on_app_focus);

    let empty_config: crate::models::PluginConfig =
        serde_json::from_value(serde_json::json!({})).unwrap();
    assert!(!empty_config.clear_notifications_on_app_focus);
}
