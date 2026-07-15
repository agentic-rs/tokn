import { defineConfig } from "vite";

export default defineConfig({
  base: "/assets/",
  build: {
    outDir: "dist",
    emptyOutDir: true,
    cssCodeSplit: false,
    rollupOptions: {
      output: {
        entryFileNames: "viewer.js",
        chunkFileNames: "viewer.js",
        assetFileNames: (asset) => (asset.name?.endsWith(".css") ? "viewer.css" : "[name][extname]")
      }
    }
  }
});
