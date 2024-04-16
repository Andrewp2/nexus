/** @type {import('tailwindcss').Config} */
module.exports = {
    content: {
        files: ["*.html", "./src/**/*.rs"],
    },
    theme: {
        extend: {
            colors: {
                'primary-color': '#4B3F72',
                'secondary-color': '#39314B',
                'background-color': '#1E1E2E',
                'accent-color': '#D1BA74',
                'hover-background-color': '#928251',
            },
            fontFamily: {
                'roboto-slab': ['"Roboto Slab"', 'serif'],
            },
            backgroundImage: {
                'special-gradient': 'linear-gradient(0 0, rgba(255, 255, 255, 0.2) 0%, rgba(255, 255, 255, 0.2) 37%, rgba(255, 255, 255, 0.8) 45%, rgba(255, 255, 255, 0.0) 50%)',
            }
        },
    },
    plugins: [],
}