import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "node:path";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  clearScreen: false,
  build: {
    rollupOptions: {
      input: {
        main: path.resolve(__dirname, "index.html"),
        // Docs viewer windows: a document or figure popped out of the Docs tab
        // (see src-tauri/src/docs_viewer.rs).
        "docs-viewer": path.resolve(__dirname, "docs-viewer.html"),
      },
    },
  },
  server: {
    port: 5173,
    strictPort: true,
  },
});
