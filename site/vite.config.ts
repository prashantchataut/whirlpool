import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// Served from https://prashantchataut.github.io/whirlpool/
export default defineConfig({
  base: "/whirlpool/",
  plugins: [react(), tailwindcss()],
});
