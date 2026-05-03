/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        surface: {
          900: "#0f1117",
          800: "#16181f",
          700: "#1e2130",
          600: "#252a3a",
          500: "#2e3450",
        },
      },
    },
  },
  plugins: [],
};
