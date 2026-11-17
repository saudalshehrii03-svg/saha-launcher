import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri expects a fixed dev port and must not clear the terminal.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  // Don't watch the Rust side: cargo locks files in src-tauri/target while building (EBUSY crash).
  server: { port: 1430, strictPort: true, watch: { ignored: ["**/src-tauri/**"] } },
  envPrefix: ["VITE_", "TAURI_ENV_"],
  build: { target: "es2022", outDir: "dist" },
});
