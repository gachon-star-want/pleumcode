import { defineConfig } from 'vite';

// https://vitejs.dev/config
export default defineConfig({
  define: {
    'process.env.GITHUB_OWNER': JSON.stringify(process.env.GITHUB_OWNER || 'gachon-star-want'),
    'process.env.GITHUB_REPO': JSON.stringify(process.env.GITHUB_REPO || 'pleum'),
    'process.env.PLEUM_BUNDLE_NAME': JSON.stringify(process.env.PLEUM_BUNDLE_NAME || 'Pleum'),
  },
});
