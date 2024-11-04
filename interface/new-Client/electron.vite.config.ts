import vue from '@vitejs/plugin-vue';
import tailwindcss from 'tailwindcss';
import autoprefixer from 'autoprefixer';
import AutoImport from 'unplugin-auto-import/vite';
import Components from 'unplugin-vue-components/vite';

import { resolve } from 'path';
import { defineConfig, externalizeDepsPlugin } from 'electron-vite';
import { NaiveUiResolver } from 'unplugin-vue-components/resolvers';

export default defineConfig({
  main: {
    plugins: [externalizeDepsPlugin()]
  },
  preload: {
    plugins: [externalizeDepsPlugin()]
  },
  renderer: {
    resolve: {
      alias: {
        '@renderer': resolve('src/renderer/src')
      }
    },
    css: {
      postcss: {
        plugins: [tailwindcss, autoprefixer]
      }
    },
    plugins: [
      vue(),
      AutoImport({
        imports: ['vue']
      }),
      Components({
        resolvers: [NaiveUiResolver()]
      })
    ],
    server: {
      proxy: {
        'http://192.168.200.8:8080/ui/': {
          target: 'http://localhost:5173',
          changeOrigin: true,
          rewrite: path => path.replace(/^\/ui/, '')
        },
        '^/static': {
          target: 'https://jumpserver-test.cmdb.cc',
          changeOrigin: true
        }
      }
    }
  }
});
