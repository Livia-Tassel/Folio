import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig, loadEnv } from "vite";

// Folio frontend is served by Tauri — we run pure SPA, no SSR.
// Tauri dev server config is driven by its own tauri.conf.json.

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, ".", "");
  const host = env.TAURI_DEV_HOST || undefined;

  return {
    plugins: [tailwindcss(), sveltekit()],

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    clearScreen: false,
    server: {
      port: 5173,
      strictPort: true,
      host: host || false,
      hmr: host
        ? {
            protocol: "ws",
            host,
            port: 5174,
          }
        : undefined,
      watch: {
        ignored: ["**/src-tauri/**"],
      },
    },
  };
});
