import type { Config } from "tailwindcss";

// Os tokens de cor abaixo vem diretamente da paleta de referencia do projeto
// (ver src/styles/theme.css para a lista comentada de onde cada tom veio).
// Mantemos os nomes semanticos (primary, accent, danger...) em vez de "teal-600"
// para que o significado de cada cor sobreviva se a paleta for ajustada depois.
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  darkMode: ["class"],
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: "#266C62",
          muted: "#508A8C",
          foreground: "#F6F2E7",
        },
        surface: {
          DEFAULT: "#FBF8F0",
          warm: "#D4C78F",
          dark: "#17221F",
        },
        accent: {
          DEFAULT: "#D2431B",
          hover: "#B5380F",
        },
        danger: {
          DEFAULT: "#87281B",
        },
        ink: {
          DEFAULT: "#1C2420",
          muted: "#5B6B66",
        },
      },
      fontFamily: {
        display: ["\"IBM Plex Sans\"", "ui-sans-serif", "system-ui", "sans-serif"],
        body: ["\"IBM Plex Sans\"", "ui-sans-serif", "system-ui", "sans-serif"],
        mono: ["\"IBM Plex Mono\"", "ui-monospace", "SFMono-Regular", "monospace"],
      },
      borderRadius: {
        sm: "4px",
        md: "6px",
        lg: "10px",
      },
      boxShadow: {
        // Propositalmente sem blur suave "SaaS card" -- usamos deslocamento
        // solido, como uma sombra de objeto fisico sobre a mesa de trabalho.
        hard: "4px 4px 0 0 rgba(28, 36, 32, 0.9)",
      },
      keyframes: {
        "progress-slide": {
          "0%": { transform: "translateX(-100%)" },
          "100%": { transform: "translateX(300%)" },
        },
      },
      animation: {
        "progress-slide": "progress-slide 1.1s ease-in-out infinite",
      },
    },
  },
  plugins: [],
} satisfies Config;
