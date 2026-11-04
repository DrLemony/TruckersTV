/// <reference types="vitest/config" />
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

// Three pages: the companion window (index.html, shown by Tauri) and the in-cabin screen
// (cabin.html, rendered offscreen by the TruckersTV host and streamed onto the cab's TV); and TV's
// player (tv-player.html), in a WebView of its own the host places over the cabin's TV screen.
/** TruckersTV's version: the workspace's (Cargo.toml), for the page outside the app (the preview). */
const VERSION = /^version = "([^"]+)"/m.exec(readFileSync(resolve(import.meta.dirname, '../Cargo.toml'), 'utf8'))?.[1]

export default defineConfig({
  define: { __WESTPLAY_VERSION__: JSON.stringify(VERSION ?? '') },
  plugins: [react()],
  clearScreen: false,
  test: {
    include: ['src/**/*.test.ts'],
    environment: 'node',
  },
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    target: 'es2023',
    rollupOptions: {
      input: {
        companion: resolve(import.meta.dirname, 'index.html'),
        cabin: resolve(import.meta.dirname, 'cabin.html'),
        tvPlayer: resolve(import.meta.dirname, 'tv-player.html'),
      },
    },
  },
})
