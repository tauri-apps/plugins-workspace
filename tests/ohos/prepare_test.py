import importlib.util
from pathlib import Path, PureWindowsPath
import subprocess
import sys
import tempfile
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('ohos_prepare', ROOT / 'shared/ohos/prepare.py')
prepare = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prepare)


class PrepareTests(unittest.TestCase):
    def test_workspace_constraints_and_windows_paths_survive_overlay(self):
        with tempfile.TemporaryDirectory() as directory:
            workspace = Path(directory)
            host = workspace / 'app'
            host.mkdir()
            (workspace / 'Cargo.toml').write_text('''[workspace]
members = ["app"]
[workspace.dependencies]
tauri = { version = "=2.11.6", features = ["protocol-asset"] }
[patch.crates-io]
other = { path = "other" }
''')
            (host / 'Cargo.toml').write_text('''[package]
name = "app"
version = "0.1.0"
[dependencies]
tauri = { workspace = true, features = ["tray-icon"] }
''')
            path = PureWindowsPath(r'C:\tools\1\tauri')
            prepare.patch_application(workspace, host, {
                'tauri': path,
                'openharmony-ability': workspace / 'ability/crates/ability',
            })
            data = tomllib.loads((workspace / 'Cargo.toml').read_text())
            dependency = data['workspace']['dependencies']['tauri']
            self.assertEqual(dependency['version'], '=2.11.6')
            self.assertEqual(dependency['path'], str(path))
            self.assertEqual(dependency['features'], ['protocol-asset'])
            self.assertEqual(data['patch']['crates-io']['other'], {'path': 'other'})
            self.assertEqual(data['patch']['crates-io']['tauri']['path'], str(path))
            dependency = tomllib.loads((host / 'Cargo.toml').read_text())['dependencies']['tauri']
            self.assertEqual(dependency, {'workspace': True, 'features': ['tray-icon']})

    def test_incompatible_fork_version_is_rejected_by_cargo(self):
        with tempfile.TemporaryDirectory() as directory:
            workspace = Path(directory)
            fork = workspace / 'fork'
            host = workspace / 'app'
            for path, name, version in ((fork, 'tauri', '2.11.5'), (host, 'app', '0.1.0')):
                (path / 'src').mkdir(parents=True)
                (path / 'src/lib.rs').write_text('')
                (path / 'Cargo.toml').write_text(f'[package]\nname = "{name}"\nversion = "{version}"\n')
            with (host / 'Cargo.toml').open('a') as output:
                output.write('[dependencies]\ntauri = "=2.11.6"\n')
            prepare.patch_application(host, host, {
                'tauri': fork,
                'openharmony-ability': workspace / 'ability/crates/ability',
            })
            result = subprocess.run(['cargo', 'metadata', '--offline', '--format-version', '1'], cwd=host,
                                    capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('=2.11.6', result.stderr)
            self.assertIn('2.11.5', result.stderr)

    def test_sources_only_preserves_application_metadata(self):
        with tempfile.TemporaryDirectory() as directory:
            host = Path(directory) / 'app'
            entry = host / 'gen/ohos/entry'
            entry.mkdir(parents=True)
            (host / 'Cargo.toml').write_text('[package]\nname = "sample"\nversion = "0.1.0"\n')
            metadata = entry / 'oh-package.json5'
            original = '// application-owned formatting\n{ "dependencies": {} }\n'
            metadata.write_text(original)
            ability = Path(directory) / 'ability'
            (ability / 'scripts').mkdir(parents=True)
            (ability / 'scripts/pack.sh').write_text('exit 0\n')
            (ability / 'ability.har').write_bytes(b'fixture HAR')
            (host / 'gen/ohos-ability-source').write_text(str(ability))
            subprocess.run([sys.executable, str(ROOT / 'shared/ohos/install.py'), str(host), '--sources-only'], check=True)
            self.assertEqual(metadata.read_text(), original)
            self.assertEqual((host / 'gen/ohos/vendor/ability.har').read_bytes(), b'fixture HAR')


if __name__ == '__main__':
    unittest.main()
