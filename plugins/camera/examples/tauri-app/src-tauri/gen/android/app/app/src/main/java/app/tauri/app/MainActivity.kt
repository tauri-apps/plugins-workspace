package app.tauri.app

import app.tauri.plugin.PluginManager

class MainActivity : TauriActivity() {
  var pluginManager: PluginManager = PluginManager(this)
}
