// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use log::RecordBuilder;

use crate::{LogLevel, WEBVIEW_TARGET};

#[tauri::command]
pub fn log(
    level: LogLevel,
    message: String,
    location: Option<&str>,
    file: Option<&str>,
    line: Option<u32>,
    key_values: Option<HashMap<String, String>>,
) {
    let level = log::Level::from(level);

    let target = if let Some(location) = location {
        format!("{WEBVIEW_TARGET}:{location}")
    } else {
        WEBVIEW_TARGET.to_string()
    };

    let mut builder = RecordBuilder::new();
    builder.level(level).target(&target).file(file).line(line);

    let key_values = key_values.unwrap_or_default();
    let mut kv = HashMap::new();
    for (k, v) in key_values.iter() {
        kv.insert(k.as_str(), v.as_str());
    }
    builder.key_values(&kv);
    #[cfg(feature = "tracing")]
    emit_trace(level, &message, location, file, line, &kv);

    log::logger().log(&builder.args(format_args!("{message}")).build());
}
