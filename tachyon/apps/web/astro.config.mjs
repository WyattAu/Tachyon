import { defineConfig } from "astro/config";
import solid from "@astrojs/solid-js";
import tailwindcss from "@tailwindcss/vite";

// Tachyon web client (ADR-113). Static output + Solid islands.
//
// SAME-ORIGIN architecture — no CORS, ever:
// - dev:  vite proxy forwards /api → TACHYON_DEV_TARGET (default staging)
// - prod: the Tachyon server serves this build from its static dir, so the
//   API is same-origin by construction. PUBLIC_TACHYON_API is only for
//   exotic deployments (e.g. CloudFlare Pages + API subdomain).
const target = process.env.TACHYON_DEV_TARGET ?? "http://192.168.1.191:18080";

export default defineConfig({
  output: "static",
  integrations: [solid()],
  vite: {
    plugins: [tailwindcss()],
    server: {
      proxy: {
        // Pass /api/* through unchanged — staging serves /api/v1/*. No rewrite.
        "/api": {
          target,
          changeOrigin: true,
        },
      },
    },
  },
});
