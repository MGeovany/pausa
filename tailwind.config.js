/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        // Raycast-inspired color palette
        primary: {
          50: "#f0f7ff",
          100: "#e0efff",
          200: "#b9dfff",
          300: "#7cc8ff",
          400: "#36b0ff",
          500: "#4E8EF7", // Main blue
          600: "#0969da",
          700: "#0550ae",
          800: "#0a4595",
          900: "#0f3a7a",
        },
        gray: {
          50: "#f9fafb",
          100: "#f3f4f6",
          200: "#e5e7eb",
          300: "#d1d5db",
          400: "#9ca3af",
          500: "#6b7280",
          600: "#4b5563",
          700: "#374151",
          800: "#1f2937",
          900: "#111827",
          950: "#0E0E10", // Dark background
        },
        surface: {
          light: "#E1E1E6", // Light surface
          dark: "#1C1C1E", // Dark surface
        },
        bg: "#0E0E10", // Background color
        text: "#E1E1E6", // Text color
        subtext: "#A1A1AA", // Subtext color
      },
      fontFamily: {
        sans: ["Inter", "SF Pro Display", "system-ui", "sans-serif"],
      },
      animation: {
        "fade-in": "fadeIn 150ms ease-out",
        "fade-out": "fadeOut 150ms ease-in",
        "scale-in": "scaleIn 150ms ease-out",
        "scale-out": "scaleOut 150ms ease-in",
        "slide-up": "slideUp 200ms ease-out",
        "slide-down": "slideDown 200ms ease-out",
      },
      keyframes: {
        fadeIn: {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        fadeOut: {
          "0%": { opacity: "1" },
          "100%": { opacity: "0" },
        },
        scaleIn: {
          "0%": { opacity: "0", transform: "scale(0.95)" },
          "100%": { opacity: "1", transform: "scale(1)" },
        },
        scaleOut: {
          "0%": { opacity: "1", transform: "scale(1)" },
          "100%": { opacity: "0", transform: "scale(0.95)" },
        },
        slideUp: {
          "0%": { opacity: "0", transform: "translateY(10px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        slideDown: {
          "0%": { opacity: "0", transform: "translateY(-10px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
      },
      backdropBlur: {
        xs: "2px",
      },
      borderRadius: {
        xl: "12px",
        "2xl": "16px",
      },
      boxShadow: {
        raycast:
          "0 0 0 1px rgba(255, 255, 255, 0.05), 0 16px 32px rgba(0, 0, 0, 0.24), 0 4px 8px rgba(0, 0, 0, 0.12)",
        widget: "0 8px 32px rgba(0, 0, 0, 0.12), 0 2px 8px rgba(0, 0, 0, 0.08)",
      },
    },
  },
  plugins: [require("@tailwindcss/forms")],
};
