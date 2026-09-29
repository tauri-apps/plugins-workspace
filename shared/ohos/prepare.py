#!/usr/bin/env python3
"""Prepare experimental sources outside application workspaces in disposable CI checkouts."""
import json
import os
from pathlib import Path
import re
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[2]
PLUGINS = ('clipboard-manager', 'dialog', 'fs', 'notification', 'opener', 'barcode-scanner')


def prepare_sources(destination):
    baseline = Path(os.environ['OHOS_TAURI_SOURCES'])
    expected = json.loads((baseline / 'tauri-harmony-pins.json').read_text())
    pin = json.loads((ROOT / 'shared/ohos/core-pin.json').read_text())
    core = Path(destination).resolve() / 'tauri'
    if core.exists():
        raise ValueError('Use a fresh external source directory for the OHOS overlay')
    subprocess.run(['git', 'init', str(core)], check=True)
    subprocess.run(['git', '-C', str(core), 'fetch', '--depth', '1', f"https://github.com/{pin['repository']}.git", pin['revision']], check=True)
    subprocess.run(['git', '-C', str(core), 'checkout', '--detach', 'FETCH_HEAD'], check=True)
    for manifest in core.rglob('Cargo.toml'):
        text = manifest.read_text().replace(
            'git = "https://github.com/harmony-contrib/openharmony-ability.git"',
            'git = "https://github.com/harmony-contrib/openharmony-ability.git", rev = "' + expected['ability']['revision'] + '"')
        manifest.write_text(text)
    manifest = core / 'Cargo.toml'
    text = manifest.read_text()
    for name in ('wry', 'tao', 'cargo-mobile2'):
        text = re.sub(rf'^{name} = .*$', f'{name} = {{ path = "{baseline / name}" }}', text, flags=re.MULTILINE)
    manifest.write_text(text)
    patches = {}
    for manifest in (core / 'crates').glob('*/Cargo.toml'):
        name = tomllib.loads(manifest.read_text()).get('package', {}).get('name', '')
        if name.startswith('tauri'):
            patches[name] = manifest.parent
    for name in ('wry', 'tao', 'cargo-mobile2'):
        patches[name] = baseline / name
    # Keep the fork's normal platforms on upstream stable Tauri. This mutation is CI-only.
    manifest = ROOT / 'Cargo.toml'
    text = manifest.read_text()
    for name in ('tauri', 'tauri-build', 'tauri-plugin', 'tauri-utils'):
        text = re.sub(rf'^{name} = .*$', f'{name} = {{ path = "{patches[name]}", default-features = false }}', text, flags=re.MULTILINE)
    manifest.write_text(text)
    for name in PLUGINS:
        patches[f'tauri-plugin-{name}'] = ROOT / 'plugins' / name
    return patches


def patch_application(workspace, host, patches):
    manifest = Path(host) / 'Cargo.toml'
    text = manifest.read_text()
    for name, path in patches.items():
        def replace_dependency(match):
            options = tomllib.loads('value = ' + match.group(1))['value']
            if isinstance(options, str):
                options = {}
            for key in ('version', 'path', 'git', 'rev', 'branch', 'tag', 'registry', 'workspace'):
                options.pop(key, None)
            options = {'path': str(path), **options}
            values = ', '.join(f'{key} = {json.dumps(value)}' for key, value in options.items())
            return name + ' = { ' + values + ' }'
        text = re.sub(rf'^{re.escape(name)} = (.*)$', replace_dependency, text, flags=re.MULTILINE)
    manifest.write_text(text)
    manifest = Path(workspace) / 'Cargo.toml'
    with manifest.open('a') as output:
        output.write('\n[patch.crates-io]\n')
        for name, path in patches.items():
            output.write(f'{name} = {{ path = "{path}" }}\n')


if __name__ == '__main__':
    import sys
    if len(sys.argv) != 4:
        raise SystemExit('prepare.py EXTERNAL_SOURCE_DIR WORKSPACE_DIR HOST_DIR')
    patch_application(sys.argv[2], sys.argv[3], prepare_sources(sys.argv[1]))
