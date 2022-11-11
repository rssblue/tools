const colors = require("tailwindcss/colors");

module.exports = {
  content: ["./index.html", "./src/**/*.rs"],
  theme: {
    fontFamily: {
      sans: ["Poppins"],
      mono: ["Source Code Pro"],
    },
    colors: {
      primary: {
          50: "#E0ECF9",
          100: "#B2D1EF",
          200: "#87B7E6",
          300: "#5F9EDC",
          400: "#3A87D3",
          500: "#1871C9",
          600: "#0F5DA9",
          700: "#08498A",
          800: "#03376A",
          900: "#00254A",
      },
      warning: {
          50: "#FFF4D4",
          100: "#FEECB8",
          200: "#FDE59D",
          300: "#FCDD81",
          400: "#FBD566",
          500: "#FBCD4B",
          600: "#D9AE31",
          700: "#B78F1B",
          800: "#95720B",
          900: "#735600",
      },
      danger: {
          50: "#F9E3E5",
          100: "#F3C6CA",
          200: "#EDAAB0",
          300: "#E78D96",
          400: "#E1717B",
          500: "#DB5461",
          600: "#C82A39",
          700: "#96202B",
          800: "#64151D",
          900: "#320B0E",
      },
      white: colors.white,
      gray: colors.gray,
      transparent: "transparent",
    },
    extend: {
      typography: {
        DEFAULT: {
          css: {
            strong: "none",
            img: "none",
            figure: "none",
            a: "none",
            code: "none",
            "code::before": {
              content: "none",
            },
            "code::after": {
              content: "none",
            },
            pre: "none",
            "pre code": {
              "white-space": "pre-wrap",
            },
          },
        },
      },
    },
  },
  plugins: [require("@tailwindcss/typography"), require("@tailwindcss/forms")],
  future: {
    hoverOnlyWhenSupported: true,
  },
};
