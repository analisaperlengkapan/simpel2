/** @type {import('tailwindcss').Config} */
module.exports = {
    content: {
        files: [
            "../../antarmuka/*/index.html",
            "../../antarmuka/*/src/**/*.rs",
            "../../lib/**/*.rs",
            "./src/**/*.rs"
        ],
    },
    theme: {
        extend: {
            colors: {
                navy: {
                    50: '#f0f4f8',
                    100: '#d9e2ec',
                    200: '#bcccdc',
                    300: '#9fb3c8',
                    400: '#829ab1',
                    500: '#627d98',
                    600: '#486581',
                    700: '#334e68',
                    800: '#1e3a5f',
                    900: '#0f172a',
                    950: '#0a1020',
                },
                gold: {
                    50: '#fefce8',
                    100: '#fef9c3',
                    200: '#fef08a',
                    300: '#fde047',
                    400: '#facc15',
                    500: '#d4a843',
                    600: '#b8860b',
                    700: '#a16207',
                    800: '#854d0e',
                    900: '#713f12',
                },
            },
        },
    },
    plugins: [],
}
