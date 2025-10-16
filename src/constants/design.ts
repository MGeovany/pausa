// Design system constants based on requirements 8.1 and 8.2

export const COLORS = {
  // Primary color palette
  primary: {
    50: '#f0f7ff',
    100: '#e0efff',
    200: '#b9dfff',
    300: '#7cc8ff',
    400: '#36b0ff',
    500: '#4E8EF7', // Main blue
    600: '#0969da',
    700: '#0550ae',
    800: '#0a4595',
    900: '#0f3a7a',
  },

  // Grayscale palette
  gray: {
    50: '#f9fafb',
    100: '#f3f4f6',
    200: '#e5e7eb',
    300: '#d1d5db',
    400: '#9ca3af',
    500: '#6b7280',
    600: '#4b5563',
    700: '#374151',
    800: '#1f2937',
    900: '#111827',
    950: '#0E0E10', // Dark background
  },

  // Surface colors
  surface: {
    light: '#E1E1E6',
    dark: '#1C1C1E',
  },

  // Semantic colors
  success: '#10b981',
  error: '#ef4444',
  warning: '#f59e0b',
  info: '#3b82f6',
} as const;

export const TYPOGRAPHY = {
  fontFamily: {
    primary: ['Inter', 'SF Pro Display', 'system-ui', 'sans-serif'],
  },
  fontSize: {
    xs: '0.75rem',    // 12px
    sm: '0.875rem',   // 14px
    base: '1rem',     // 16px
    lg: '1.125rem',   // 18px
    xl: '1.25rem',    // 20px
    '2xl': '1.5rem',  // 24px
    '3xl': '1.875rem', // 30px
    '4xl': '2.25rem', // 36px
  },
  fontWeight: {
    normal: 400,
    medium: 500,
    semibold: 600,
    bold: 700,
  },
} as const;

export const SPACING = {
  xs: '0.25rem',   // 4px
  sm: '0.5rem',    // 8px
  md: '0.75rem',   // 12px
  lg: '1rem',      // 16px
  xl: '1.5rem',    // 24px
  '2xl': '2rem',   // 32px
  '3xl': '3rem',   // 48px
  '4xl': '4rem',   // 64px
} as const;

export const BORDER_RADIUS = {
  sm: '0.375rem',   // 6px
  md: '0.5rem',     // 8px
  lg: '0.75rem',    // 12px
  xl: '0.75rem',    // 12px
  '2xl': '1rem',    // 16px
  full: '9999px',
} as const;

export const SHADOWS = {
  sm: '0 1px 2px 0 rgb(0 0 0 / 0.05)',
  md: '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
  lg: '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
  xl: '0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)',
  raycast: '0 0 0 1px rgba(255, 255, 255, 0.05), 0 16px 32px rgba(0, 0, 0, 0.24), 0 4px 8px rgba(0, 0, 0, 0.12)',
  widget: '0 8px 32px rgba(0, 0, 0, 0.12), 0 2px 8px rgba(0, 0, 0, 0.08)',
} as const;

export const ANIMATIONS = {
  duration: {
    fast: '100ms',
    normal: '150ms',
    slow: '200ms',
  },
  easing: {
    easeOut: 'cubic-bezier(0, 0, 0.2, 1)',
    easeIn: 'cubic-bezier(0.4, 0, 1, 1)',
    easeInOut: 'cubic-bezier(0.4, 0, 0.2, 1)',
  },
} as const;

export const Z_INDEX = {
  dropdown: 1000,
  sticky: 1020,
  fixed: 1030,
  modal: 1040,
  popover: 1050,
  tooltip: 1060,
  toast: 1070,
  overlay: 1080,
} as const;

// Component-specific constants
export const FOCUS_WIDGET = {
  defaultSize: {
    width: 200,
    height: 60,
  },
  minSize: {
    width: 160,
    height: 50,
  },
  progressRingSize: 32,
  progressRingStroke: 3,
} as const;

export const COMMAND_PALETTE = {
  maxWidth: 600,
  maxHeight: 400,
  itemHeight: 48,
  maxVisibleItems: 8,
} as const;

export const BREAK_OVERLAY = {
  backdropBlur: '8px',
  contentMaxWidth: 480,
} as const;

export const HOTKEYS = {
  commandPalette: 'cmd+space',
  toggleFocus: 'cmd+shift+f',
  lockNow: 'cmd+shift+l',
} as const;
