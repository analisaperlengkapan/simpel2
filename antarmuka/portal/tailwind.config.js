/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.rs",
    "./index.html",
    "../../lib/ui/src/**/*.rs",
  ],
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
          800: '#1e3a5f', // Main navy
          900: '#102a43',
          950: '#061523',
        },
        // `primary` IS navy. The frontends have always written `bg-primary-600`,
        // `ring-primary-500`, `text-primary-700` … (~100 uses, most of them in
        // the shared lib/ui crate) but neither config defined the scale, so
        // Tailwind emitted nothing: primary buttons rendered transparent and
        // focus rings never appeared. It is defined as an alias of the navy
        // scale above; `check-color-contrast.py` fails if the two drift apart.
        primary: {
          DEFAULT: '#1e3a5f', // = navy-800, for bare `bg-primary` / `text-primary`
          dark: '#102a43', // = navy-900, for `hover:bg-primary-dark`
          50: '#f0f4f8',
          100: '#d9e2ec',
          200: '#bcccdc',
          300: '#9fb3c8',
          400: '#829ab1',
          500: '#627d98',
          600: '#486581',
          700: '#334e68',
          800: '#1e3a5f',
          900: '#102a43',
          950: '#061523',
        },
        gold: {
          50: '#fbf7e6',
          100: '#f5ebc5',
          200: '#edd996',
          300: '#e3c162',
          400: '#d4a843', // Main gold
          500: '#b88d2d',
          600: '#926d1f',
          700: '#705115',
          800: '#523a0f',
          900: '#36260a',
          950: '#201605',
        },
      },
      // Centralized z-index layers — every overlay (dropdown, popover,
      // tooltip, modal, toast) picks one of these instead of hardcoding
      // numeric values. Mirrored in antarmuka/perlengkapan/tailwind.config.js.
      zIndex: {
        'dropdown': '40',
        'popover':  '60',
        'tooltip':  '70',
        'modal':    '80',
        'toast':    '90',
      },
    },
  },
  plugins: [],
}