#!/usr/bin/env python3
"""Configure the sample's unsigned HAP; application projects own their release metadata."""
from pathlib import Path
import json
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
s = p.read_text()
needle = '"--target", target.toString()]'
if needle not in s:
    raise ValueError('Upstream release Rust callback template changed')
p.write_text(s.replace(needle, '"--target", target.toString(), "--release"]'))
