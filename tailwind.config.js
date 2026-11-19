/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ['class'],
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        muted: {
          foreground: '#6b7280',
        },
      },
    },
  },
  plugins: [],
};