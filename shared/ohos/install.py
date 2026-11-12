#!/usr/bin/env python3
"""Install native plugin glue into a freshly generated OHOS application."""
import json
import argparse
from pathlib import Path
import shutil
import subprocess
import sys
import tomllib
import json5

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('host')
parser.add_argument('--without-barcode', action='store_true', help='Omit ScanKit when building for OpenHarmony without Huawei ScanKit')
parser.add_argument('--sources-only', action='store_true', help='Keep the application-owned project metadata and Ability unchanged')
args = parser.parse_args()
source = Path(__file__).resolve().parent
host = Path(args.host).resolve()
manifest = tomllib.loads((host / 'Cargo.toml').read_text())
library = manifest.get('lib', {}).get('name', manifest['package']['name'].replace('-', '_'))
project = host / 'gen/ohos'
entry = project / 'entry'
destination = entry / 'src/main/ets/tauri-plugins'
ignored = ['__pycache__', '*.pyc', '*.py', '*.json', '*.md'] + (['Barcode.ets'] if args.without_barcode else [])
shutil.copytree(source, destination, ignore=shutil.ignore_patterns(*ignored), dirs_exist_ok=True)
if args.without_barcode:
    bridge = destination / 'TauriPlugins.ets'
    text = bridge.read_text().replace("import { Barcode } from './plugins/Barcode';\n", '').replace("    this.plugins.set('barcode-scanner', new Barcode(context));\n", '')
    bridge.write_text(text)
module_name = f'lib{library}.so'
types = entry / 'src/main/cpp/types/tauri-plugins-native'
types.mkdir(parents=True, exist_ok=True)
(types / 'oh-package.json5').write_text(json.dumps({'name': module_name, 'version': '1.0.0', 'types': './index.d.ts'}, indent=2)+'\n')
(types / 'index.d.ts').write_text('''interface NativeModule {
  tauriOhosPluginInitialize(callback: (request: string) => void, files: string, cache: string, temp: string): void;
  tauriOhosPluginResponse(id: number, success: boolean, payload: string): void;
  tauriOhosPluginClose(): void;
}
declare const native: NativeModule;
export default native;
''')
ability_source = Path((host / 'gen/ohos-ability-source').read_text().strip())
subprocess.run(['bash', 'scripts/pack.sh'], cwd=ability_source, check=True)
archives = list(ability_source.glob('*.har'))
if len(archives) != 1:
    raise ValueError(f'Expected one Ability HAR from pinned sources, got {archives}')
vendor = project / 'vendor'
vendor.mkdir(exist_ok=True)
shutil.copyfile(archives[0], vendor / 'ability.har')
if args.sources_only:
    print(f'Installed pinned HAR and plugin sources into {project}')
    sys.exit(0)

package = entry / 'oh-package.json5'
data = json5.loads(package.read_text())
data.setdefault('dependencies', {})[module_name] = 'file:./src/main/cpp/types/tauri-plugins-native'
data['dependencies']['@ohos-rs/ability'] = 'file:../vendor/ability.har'
package.write_text(json.dumps(data, indent=2)+'\n')

ability = entry / 'src/main/ets/entryability/EntryAbility.ets'
text = ability.read_text()
if 'TauriPlugins' in text:
    raise ValueError('Native glue already installed; generate a fresh OHOS project')
text = f"import native from '{module_name}'\nimport {{ TauriPlugins }} from '../tauri-plugins/TauriPlugins'\nimport {{ NativeModule }} from '../tauri-plugins/NativePlugin'\n" + text
adapter = """class TauriNativeModule implements NativeModule {
  tauriOhosPluginInitialize(callback: (request: string) => void, files: string, cache: string, temp: string): void {
    native.tauriOhosPluginInitialize(callback, files, cache, temp);
  }
  tauriOhosPluginResponse(id: number, success: boolean, payload: string): void {
    native.tauriOhosPluginResponse(id, success, payload);
  }
  tauriOhosPluginClose(): void { native.tauriOhosPluginClose(); }
}
"""
text = text.replace('export default class EntryAbility extends RustAbility {', adapter + 'export default class EntryAbility extends RustAbility {\n  private tauriPlugins?: TauriPlugins;')
needle = 'super.onCreate(want, launchParam);'
if needle not in text:
    raise ValueError('The generated Ability onCreate template changed')
text = text.replace(needle, 'this.tauriPlugins = new TauriPlugins(this.context, new TauriNativeModule());\n    await super.onCreate(want, launchParam);')
end = text.rindex('}')
text = text[:end]+'''  onDestroy(): void {
    this.tauriPlugins?.close();
    super.onDestroy();
  }
'''+text[end:]
ability.write_text(text)
module = entry / 'src/main/module.json5'
data = json5.loads(module.read_text())
permissions = data['module'].setdefault('requestPermissions', [])
strings = entry / 'src/main/resources/base/element/string.json'
resources = json5.loads(strings.read_text())
for name, reason, text in [
    ('ohos.permission.CAMERA', 'tauri_camera_reason', 'Scan QR codes and barcodes'),
    ('ohos.permission.READ_PASTEBOARD', 'tauri_clipboard_reason', 'Read text you choose to paste into the application'),
]:
    if args.without_barcode and name == 'ohos.permission.CAMERA':
        continue
    if not any(p['name'] == name for p in permissions):
        permissions.append({'name': name, 'reason': f'$string:{reason}', 'usedScene': {'abilities': ['EntryAbility'], 'when': 'inuse'}})
    resources['string'].append({'name': reason, 'value': text})
module.write_text(json.dumps(data, indent=2)+'\n')
strings.write_text(json.dumps(resources, indent=2)+'\n')
print(f'Installed native plugin glue into {project}')
