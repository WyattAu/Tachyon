export default {
  plugins: {
    '@tailwindcss/postcss': {
      // Content paths for class detection
      content: [
        './index.html',
        './src/**/*.{ts,js,html}',
      ],
    },
    autoprefixer: {},
  },
};
