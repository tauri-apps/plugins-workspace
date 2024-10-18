// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { defineConfig } from 'vite'
import Unocss from 'unocss/vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import process from 'process'

const host = process.env.TAURI_DEV_HOST

// https://vitejs.dev/config/
export default defineConfig(async () => {
  return {
    plugins: [Unocss(), svelte()],
    build: {
      rollupOptions: {
        output: {
          entryFileNames: `assets/[name].js`,
          chunkFileNames: `assets/[name].js`,
          assetFileNames: `assets/[name].[ext]`
        }
      }
    },
    server: {
      host: host || false,
      port: 5173,
      strictPort: true,
      fs: {
        allow: ['.', '../../tooling/api/dist']
      }
    }
  }
})
