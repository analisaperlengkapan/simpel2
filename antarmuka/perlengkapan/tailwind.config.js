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
        // Kejaksaan navy surface palette
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
          900: '#102a43',
          950: '#061523',
        },
        // Kejaksaan gold accent
        gold: {
          50: '#fbf7e6',
          100: '#f5ebc5',
          200: '#edd996',
          300: '#e3c162',
          400: '#d4a843',
          500: '#b88d2d',
          600: '#926d1f',
          700: '#705115',
          800: '#523a0f',
          900: '#36260a',
          950: '#201605',
        },
        // Semantic surface tokens used by PageLayout/SectionCard/StatCard.
        // Resolve to navy shades so the whole app shares one palette.
        surface: {
          DEFAULT: '#0f172a',
          raised: '#111c36',
          sunken:  '#0a1020',
          panel:   '#0c1425',
          border:  'rgba(148, 163, 184, 0.14)',
          muted:   'rgba(148, 163, 184, 0.06)',
        },
        // Semantic status tokens (matching Tailwind defaults, named for
        // readability and so contract tests can enforce "no hex colors").
        //
        // The scales must stay CONTIGUOUS over what the tree actually uses.
        // 200 and 300 were missing while 115 classes referenced them —
        // `text-warning-300`, `text-danger-300` and friends — and Tailwind
        // generates nothing for a shade it does not know, so the text simply
        // inherited whatever colour was around it. Every status message in
        // perlengkapan was rendering in plain slate on a correctly-coloured
        // background. `check-tailwind-shades-exist.py` now derives the used
        // set from the content globs and fails on the next gap.
        success: {
          50:  '#ecfdf5',
          100: '#d1fae5',
          200: '#a7f3d0',
          300: '#6ee7b7',
          400: '#34d399',
          500: '#10b981',
          600: '#059669',
          700: '#047857',
          900: '#064e3b',
        },
        warning: {
          50:  '#fffbeb',
          100: '#fef3c7',
          200: '#fde68a',
          300: '#fcd34d',
          400: '#fbbf24',
          500: '#f59e0b',
          600: '#d97706',
          700: '#b45309',
          900: '#78350f',
        },
        danger: {
          50:  '#fef2f2',
          100: '#fee2e2',
          200: '#fecaca',
          300: '#fca5a5',
          400: '#f87171',
          500: '#ef4444',
          600: '#dc2626',
          700: '#b91c1c',
          900: '#7f1d1d',
        },
        info: {
          50:  '#eff6ff',
          100: '#dbeafe',
          200: '#bfdbfe',
          300: '#93c5fd',
          400: '#60a5fa',
          500: '#3b82f6',
          600: '#2563eb',
          700: '#1d4ed8',
          900: '#1e3a8a',
        },
      },
      fontFamily: {
        sans: [
          'Inter',
          'Segoe UI',
          'system-ui',
          '-apple-system',
          'sans-serif',
        ],
      },
      fontSize: {
        '2xs': ['0.68rem', { lineHeight: '1rem' }],
      },
      boxShadow: {
        'card':    '0 1px 2px rgba(0, 0, 0, 0.25), 0 4px 12px rgba(0, 0, 0, 0.35)',
        'panel':   '0 24px 64px rgba(0, 0, 0, 0.55)',
        'overlay': '0 32px 96px rgba(0, 0, 0, 0.70)',
      },
      borderRadius: {
        'xl2': '1.125rem',
      },
      backgroundImage: {
        'app-gradient': 'linear-gradient(180deg, #0f172a 0%, #111c36 50%, #0a1020 100%)',
        'gold-gradient': 'linear-gradient(135deg, #d4a843 0%, #facc15 100%)',
      },
      // Centralized z-index layers — every overlay (dropdown, popover,
      // tooltip, modal, toast) picks one of these instead of hardcoding
      // numeric values. Each layer leaves room above/below for ad-hoc
      // adjustments without colliding with the next named layer.
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
