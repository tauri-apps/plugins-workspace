#!/usr/bin/env python3
"""Configure the sample's unsigned HAP; application projects own their release metadata."""
from pathlib import Path
import json
import os
import json5

project = Path('examples/ohos/src-tauri/gen/ohos')
profile = project / 'build-profile.json5'
data = json5.loads(profile.read_text())
data['app']['signingConfigs'] = []
for product in data['app']['products']:
    product.pop('signingConfig', None)
    product['compatibleSdkVersion'] = '6.0.0(20)'
    product['compileSdkVersion'] = '6.0.0(20)'
profile.write_text(json.dumps(data, indent=2)+'\n')
p = project / 'entry/hvigorfile.ts'
# CLI owns native compilation, including the target and release profile.
p.write_text("import { hapTasks } from '@ohos/hvigor-ohos-plugin';\nexport default { system: hapTasks, plugins: [] };\n")

p = project / 'entry/build-profile.json5'
data = json5.loads(p.read_text())
data.setdefault('buildOption', {}).setdefault('externalNativeOptions', {})['abiFilters'] = [os.environ['OHOS_ARCH']]
p.write_text(json.dumps(data, indent=2)+'\n')
