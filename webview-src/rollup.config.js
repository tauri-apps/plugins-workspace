import resolve from '@rollup/plugin-node-resolve'
import { terser } from 'rollup-plugin-terser'
import typescript from '@rollup/plugin-typescript'

export default {
	input: './webview-src/index.ts',
	output: {
		dir: './webview-dist',
		entryFileNames: '[name].js',
		format: 'es',
		exports: 'auto'
	},
	plugins: [
    resolve(),
		terser(),
		typescript({
				tsconfig: './webview-src/tsconfig.json'
			})
		]
}
