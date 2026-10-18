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
    strictPort: true,
    proxy: {
      '/api/v1/agent': 'http://127.0.0.1:3003',
      '/api/auth/me': 'http://127.0.0.1:3001',
      '/api/auth': 'http://127.0.0.1:3002',
      '/api': 'http://127.0.0.1:3001',
      '/auth/callback': 'http://127.0.0.1:3001',
      '/auth/set-session': 'http://127.0.0.1:3001',
      '/auth/logout': 'http://127.0.0.1:3001',
      '/resume': 'http://127.0.0.1:3001',
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
        functions: 75,  // TODO: Raise to 80 after adding SSE streaming integration tests (useAgentStream.ts)
        lines: 90,
      },
    },
  },
})
