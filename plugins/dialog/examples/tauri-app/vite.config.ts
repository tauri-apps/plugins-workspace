import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { internalIpV4 } from "internal-ip";

// https://vitejs.dev/config/
export default defineConfig(async () => {
  const host =
    process.env.TAURI_PLATFORM === "android" ||
    process.env.TAURI_PLATFORM === "ios"
      ? await internalIpV4()
      : "localhost";
  return {
    plugins: [svelte()],

    // Vite optons tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    // prevent vite from obscuring rust errors
    clearScreen: false,
    // tauri expects a fixed port, fail if that port is not available
    server: {
      host: "0.0.0.0",
      port: 5173,
      strictPort: true,
      hmr: {
        protocol: "ws",
        host,
        port: 5183,
      },
      fs: {
        allow: [".", "../../tooling/api/dist"],
      },
    },
    // to make use of `TAURI_DEBUG` and other env variables
    // https://tauri.studio/v1/api/config#buildconfig.beforedevcommand
    envPrefix: ["VITE_", "TAURI_"],
    build: {
      // Tauri supports es2021
      target: ["es2021", "chrome100", "safari13"],
      // don't minify for debug builds
      minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
      // produce sourcemaps for debug builds
      sourcemap: !!process.env.TAURI_DEBUG,
    },
  };
});
