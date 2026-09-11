/** Tailwind AOT config — extracted from index.html's inline Play-CDN config.
 * Build: bunx tailwindcss@3.4 -i input.css -o public/app.css --minify
 */
const plugin = require('tailwindcss/plugin');

/** @type {import('tailwindcss').Config} */
module.exports = {
    darkMode: 'class',
    content: [
        './src/**/*.rs',
        './index.html',
    ],
    theme: {
        extend: {
            fontFamily: {
                sans: ['Inter', '-apple-system', 'BlinkMacSystemFont', 'sans-serif'],
                mono: ['JetBrains Mono', 'Fira Code', 'monospace'],
                display: ['Inter', '-apple-system', 'BlinkMacSystemFont', 'sans-serif'],
            },
            borderRadius: {
                'amoebic': '60% 40% 30% 70% / 60% 30% 70% 40%',
                'amoebic-alt': '30% 70% 70% 30% / 30% 30% 70% 70%',
                'amoebic-sm': '40% 60% 50% 50% / 50% 40% 60% 50%',
            },
            boxShadow: {
                'spatial': '4px 4px 0px 0px',
                'spatial-lg': '8px 8px 0px 0px',
                'spatial-xl': '12px 12px 0px 0px',
                'spatial-sm': '2px 2px 0px 0px',
                'spatial-accent': '4px 4px 0px 0px',
            },
            transitionTimingFunction: {
                'spring': 'cubic-bezier(0.34, 1.56, 0.64, 1)',
                'spring-bounce': 'cubic-bezier(0.175, 0.885, 0.32, 1.275)',
                'spatial': 'cubic-bezier(0.25, 0.46, 0.45, 0.94)',
            },
        },
    },
    plugins: [
        // Prose classes for document rendering
        plugin(function ({ addUtilities }) {
            const newUtilities = {
                '.prose': {
                    'color': '#374151',
                    'line-height': '1.75',
                    'max-width': '65ch',
                },
                '.prose-sm': {
                    'font-size': '0.875rem',
                    'line-height': '1.7142857',
                },
            };
            addUtilities(newUtilities);
        }),
    ],
};
