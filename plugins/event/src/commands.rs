use serde::{Deserialize, Deserializer};
use serde_json::Value as JsonValue;
use tauri::{api::ipc::CallbackFn, Manager, Runtime, Window};

pub struct EventId(String);

impl<'de> Deserialize<'de> for EventId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let event_id = String::deserialize(deserializer)?;
        if is_event_name_valid(&event_id) {
            Ok(EventId(event_id))
        } else {
            Err(serde::de::Error::custom(
                "Event name must include only alphanumeric characters, `-`, `/`, `:` and `_`.",
            ))
        }
    }
}
fn is_event_name_valid(event: &str) -> bool {
    event
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '/' || c == ':' || c == '_')
}

#[tauri::command]
pub fn listen<R: Runtime>(
    window: Window<R>,
    event: EventId,
    handler: CallbackFn,
) -> tauri::Result<usize> {
    window.listen_js(None, event.0, handler)
}

#[tauri::command]
pub fn unlisten<R: Runtime>(
    window: Window<R>,
    event: EventId,
    event_id: usize,
) -> tauri::Result<()> {
    window.unlisten_js(event.0, event_id)
}

#[tauri::command]
pub fn emit<R: Runtime>(
    window: Window<R>,
    event: EventId,
    payload: Option<JsonValue>,
) -> tauri::Result<()> {
    // dispatch the event to Rust listeners
    window.trigger(
        &event.0,
        payload.as_ref().and_then(|p| {
            serde_json::to_string(&p)
                .map_err(|e| {
                    #[cfg(debug_assertions)]
                    eprintln!("{e}");
                    e
                })
                .ok()
        }),
    );

    // emit event to JS
    window.emit_all(&event.0, payload)
}
