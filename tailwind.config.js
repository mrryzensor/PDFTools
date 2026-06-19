/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{svelte,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        "surface-container": "#1e1f25",
        "background": "#0d0e12",
        "primary": "#4f46e5", // Indigo accent
        "primary-light": "#818cf8",
        "secondary": "#10b981", // Emerald accent
        "on-surface": "#e2e8f0",
        "on-surface-variant": "#94a3b8",
        "outline-variant": "#334155",
      },
      backdropBlur: {
        xs: '2px',
      }
    },
  },
  plugins: [],
}
