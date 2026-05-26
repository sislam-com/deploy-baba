import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  base: '/',
  build: {
    outDir: 'dist',
    rollupOptions: {
      output: {
        assetFileNames: 'assets/[name]-[hash][extname]',
        chunkFileNames: 'assets/[name]-[hash].js',
        entryFileNames: 'assets/[name]-[hash].js',
      },
    },
  },
  server: {
    port: 3000,
    proxy: {
      '/api/v1/agent': 'http://localhost:3003',
      '/api/auth/me': 'http://localhost:3001',
      '/api/auth': 'http://localhost:3002',
      '/api': 'http://localhost:3001',
      '/auth/callback': 'http://localhost:3001',
      '/auth/set-session': 'http://localhost:3001',
      '/auth/logout': 'http://localhost:3001',
      '/resume': 'http://localhost:3001',
    },
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['src/test/setup.ts'],
    coverage: {
      reporter: ['text', 'lcov'],
      exclude: [
        // Config / build files
        'postcss.config.*',
        'tailwind.config.*',
        'vite.config.*',
        'eslint.config.*',
        // Build output
        'dist/**',
        // Generated code
        'src/api/types.gen.ts',
        // Test infrastructure (setup, utils, mocks)
        'src/test/**',
        // Duplicate generated file at root
        'web/**',
        // Entry point (side-effect render, not testable in unit tests)
        'src/main.tsx',
      ],
      thresholds: {
        statements: 90,
        branches: 80,
        functions: 80,
        lines: 90,
      },
    },
  },
})
