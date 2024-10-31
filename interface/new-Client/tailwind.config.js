/** @type {import('tailwindcss').Config} */

module.exports = {
  content: ['./src/renderer/**/*.{js,jsx,ts,tsx,vue}'],
  theme: {
    extend: {
      backgroundColor: {
        primary: 'rgb(var(--el-color-background) / <alpha-value>)',
        secondary: 'rgb(var(--el-color-secondary-background) / <alpha-value>)'
      },
      textColor: {
        primary: 'rgb(var(--el-color-text) / <alpha-value>)'
      },
      borderColor: {
        primary: 'rgb(var(--el-border-color) / <alpha-value>)'
      }
    }
  },
  plugins: []
};
