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
      },
      colors: {
        primary: {
          100: 'rgba(var(--primary-color), 0.1)',
          900: 'rgba(var(--primary-color), 0.2)'
        }
      },
      fontFamily: {
        custom: ['YourFontName', 'sans-serif']
      }
    }
  },
  plugins: []
};
