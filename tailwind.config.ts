import type { Config } from "tailwindcss";

export default {
  content: [
    "./app/**/*.{vue,js,ts,jsx,tsx,html}",
    "./components/**/*.{vue,js,ts}",
    "./layouts/**/*.vue",
    "./pages/**/*.vue",
  ],
} satisfies Config; 
