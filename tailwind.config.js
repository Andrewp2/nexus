/** @type {import('tailwindcss').Config} */
module.exports = {
    content: {
        relative: true,
        files: ["*.html", "./src/**/*.rs", "./app/src/**/*.rs"],
    },
    theme: {
        extend: {
            colors: {
                'primary-color': '#4F0C97',
                'secondary-color': '#7D275C',
                'background-color': '#1A1320',
                'accent-color': '#C54671',
                'hover-accent-color': '#C54671',
                't-color': '#E8DBF4',
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
    safelist: [
        "bg-red-400",
        "bg-yellow-400",
        "bg-green-400"
    ]
}