import { TanStackRouterVite } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
// import { visualizer } from "rollup-plugin-visualizer";
import type { Plugin } from "vite";
import { defineConfig } from "vite";
import compression from "vite-plugin-compression";
import { VitePWA } from "vite-plugin-pwa";

const handlePureAnnotations: Plugin = {
  name: "handle-pure-annotations",
  transform(code: string, id: string) {
    if (id.includes("@privy-io/react-auth")) {
      return {
        code: code.replace(/\/\*#__PURE__\*\//g, ""),
        map: null,
      };
    }
  },
};

export default defineConfig({
  plugins: [
    handlePureAnnotations,
    TanStackRouterVite(),
    react(),
    VitePWA({
      registerType: "autoUpdate",
      manifest: {
        name: "listen",
        short_name: "listen",
        theme_color: "#A855F7",
        icons: [
          {
            src: "/listen-more.png",
            sizes: "192x192",
            type: "image/png",
          },
        ],
      },
      workbox: {
        maximumFileSizeToCacheInBytes: 4 * 1024 * 1024,
      },
    }),
    // 暂时注释掉 visualizer
    // visualizer(),
    compression(),
  ],
  build: {
    target: "esnext",
    minify: "esbuild",
    sourcemap: false,
    chunkSizeWarningLimit: 1000,
    // rollupOptions: {
    //   output: {
    //     manualChunks(id) {
    //       if (id.includes("node_modules")) {
    //         if (id.includes("react")) {
    //           return "vendor-react";
    //         }
    //         if (id.includes("@privy-io")) {
    //           return "vendor-privy";
    //         }
    //         if (id.includes("@coinbase")) {
    //           return "vendor-coinbase";
    //         }
    //         if (id.includes("viem")) {
    //           return "vendor-viem";
    //         }
    //         if (id.includes("@walletconnect")) {
    //           return "vendor-walletconnect";
    //         }
    //         // if (id.includes("@ethersproject")) {
    //         //   return "vendor-etheresproject";
    //         // }
    //         if (id.includes("@noble")) {
    //           return "vendor-noble";
    //         }
    //         if (id.includes("lodash")) {
    //           return "vendor-lodash";
    //         }
    //         if (id.includes("@solana")) {
    //           return "vendor-solana";
    //         }
    //         if (id.includes("@tanstack")) {
    //           return "vendor-tanstack";
    //         }
    //         return "vendor";
    //       }
    //     },
    //   },
    // },
  },
});
