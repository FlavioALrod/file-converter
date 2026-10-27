import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Configuracao do Vite recomendada pelo proprio time do Tauri:
// https://tauri.app/start/frontend/vite/
// - server.port fixo e strictPort, porque o processo Rust espera essa porta.
// - ignora a pasta src-tauri no watcher, senao cada `cargo build` dispara reload.
export default defineConfig(async () => ({
  plugins: [react()],

  // Espelha o alias "@/*" definido em tsconfig.json, para "@/lib/..." etc.
  // funcionarem tanto na checagem de tipos quanto no bundle real do Vite.
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },

  // Evita que o Vite tente usar um navegador especifico; o Tauri controla a janela.
  clearScreen: false,

  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },

  envPrefix: ["VITE_", "TAURI_"],

  build: {
    // Tauri usa Chromium no Windows e WebKit no Linux/macOS; manter um alvo
    // razoavelmente moderno evita polyfills desnecessarios.
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari13",
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
}));
