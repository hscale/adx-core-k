export interface ThemeConfig {
  colors: {
    primary: ColorScale;
    secondary: ColorScale;
    success: ColorScale;
    warning: ColorScale;
    error: ColorScale;
    neutral: ColorScale;
    background: BackgroundColors;
    surface: SurfaceColors;
    text: TextColors;
    border: BorderColors;
  };
  spacing: SpacingScale;
  borderRadius: BorderRadiusScale;
  fontSize: FontSizeScale;
  fontWeight: FontWeightScale;
  transitions: TransitionScale;
  shadows: ShadowScale;
  zIndex: ZIndexScale;
}

export interface ColorScale {
  50: string;
  100: string;
  200: string;
  300: string;
  400: string;
  500: string;
  600: string;
  700: string;
  800: string;
  900: string;
  950: string;
}

export interface BackgroundColors {
  default: string;
  secondary: string;
  tertiary: string;
  accent: string;
}

export interface SurfaceColors {
  default: string;
  secondary: string;
  tertiary: string;
}

export interface TextColors {
  primary: string;
  secondary: string;
  tertiary: string;
  inverse: string;
}

export interface BorderColors {
  default: string;
  secondary: string;
  accent: string;
}

export interface SpacingScale {
  xs: string;
  sm: string;
  md: string;
  lg: string;
  xl: string;
  '2xl': string;
}

export interface BorderRadiusScale {
  sm: string;
  default: string;
  md: string;
  lg: string;
  xl: string;
}

export interface FontSizeScale {
  xs: string;
  sm: string;
  base: string;
  lg: string;
  xl: string;
  '2xl': string;
  '3xl': string;
  '4xl': string;
}

export interface FontWeightScale {
  normal: string;
  medium: string;
  semibold: string;
  bold: string;
}

export interface TransitionScale {
  fast: string;
  normal: string;
  slow: string;
}

export interface ShadowScale {
  sm: string;
  default: string;
  md: string;
  lg: string;
  xl: string;
}

export interface ZIndexScale {
  dropdown: string;
  sticky: string;
  fixed: string;
  modalBackdrop: string;
  modal: string;
  popover: string;
  tooltip: string;
  toast: string;
}

// Default theme configuration
export const defaultThemeConfig: ThemeConfig = {
  colors: {
    primary: {
      50: 'rgb(239 246 255)',
      100: 'rgb(219 234 254)',
      200: 'rgb(191 219 254)',
      300: 'rgb(147 197 253)',
      400: 'rgb(96 165 250)',
      500: 'rgb(59 130 246)',
      600: 'rgb(37 99 235)',
      700: 'rgb(29 78 216)',
      800: 'rgb(30 64 175)',
      900: 'rgb(30 58 138)',
      950: 'rgb(23 37 84)',
    },
    secondary: {
      50: 'rgb(248 250 252)',
      100: 'rgb(241 245 249)',
      200: 'rgb(226 232 240)',
      300: 'rgb(203 213 225)',
      400: 'rgb(148 163 184)',
      500: 'rgb(100 116 139)',
      600: 'rgb(71 85 105)',
      700: 'rgb(51 65 85)',
      800: 'rgb(30 41 59)',
      900: 'rgb(15 23 42)',
      950: 'rgb(2 6 23)',
    },
    success: {
      50: 'rgb(240 253 244)',
      100: 'rgb(220 252 231)',
      200: 'rgb(187 247 208)',
      300: 'rgb(134 239 172)',
      400: 'rgb(74 222 128)',
      500: 'rgb(34 197 94)',
      600: 'rgb(22 163 74)',
      700: 'rgb(21 128 61)',
      800: 'rgb(22 101 52)',
      900: 'rgb(20 83 45)',
      950: 'rgb(5 46 22)',
    },
    warning: {
      50: 'rgb(255 251 235)',
      100: 'rgb(254 243 199)',
      200: 'rgb(253 230 138)',
      300: 'rgb(252 211 77)',
      400: 'rgb(251 191 36)',
      500: 'rgb(245 158 11)',
      600: 'rgb(217 119 6)',
      700: 'rgb(180 83 9)',
      800: 'rgb(146 64 14)',
      900: 'rgb(120 53 15)',
      950: 'rgb(69 26 3)',
    },
    error: {
      50: 'rgb(254 242 242)',
      100: 'rgb(254 226 226)',
      200: 'rgb(254 202 202)',
      300: 'rgb(252 165 165)',
      400: 'rgb(248 113 113)',
      500: 'rgb(239 68 68)',
      600: 'rgb(220 38 38)',
      700: 'rgb(185 28 28)',
      800: 'rgb(153 27 27)',
      900: 'rgb(127 29 29)',
      950: 'rgb(69 10 10)',
    },
    neutral: {
      50: 'rgb(250 250 250)',
      100: 'rgb(245 245 245)',
      200: 'rgb(229 229 229)',
      300: 'rgb(212 212 212)',
      400: 'rgb(163 163 163)',
      500: 'rgb(115 115 115)',
      600: 'rgb(82 82 82)',
      700: 'rgb(64 64 64)',
      800: 'rgb(38 38 38)',
      900: 'rgb(23 23 23)',
      950: 'rgb(10 10 10)',
    },
    background: {
      default: 'rgb(var(--color-background))',
      secondary: 'rgb(var(--color-background-secondary))',
      tertiary: 'rgb(var(--color-background-tertiary))',
      accent: 'rgb(var(--color-background-accent))',
    },
    surface: {
      default: 'rgb(var(--color-surface))',
      secondary: 'rgb(var(--color-surface-secondary))',
      tertiary: 'rgb(var(--color-surface-tertiary))',
    },
    text: {
      primary: 'rgb(var(--color-text-primary))',
      secondary: 'rgb(var(--color-text-secondary))',
      tertiary: 'rgb(var(--color-text-tertiary))',
      inverse: 'rgb(var(--color-text-inverse))',
    },
    border: {
      default: 'rgb(var(--color-border))',
      secondary: 'rgb(var(--color-border-secondary))',
      accent: 'rgb(var(--color-border-accent))',
    },
  },
  spacing: {
    xs: '0.25rem',
    sm: '0.5rem',
    md: '1rem',
    lg: '1.5rem',
    xl: '2rem',
    '2xl': '3rem',
  },
  borderRadius: {
    sm: '0.25rem',
    default: '0.375rem',
    md: '0.5rem',
    lg: '0.75rem',
    xl: '1rem',
  },
  fontSize: {
    xs: '0.75rem',
    sm: '0.875rem',
    base: '1rem',
    lg: '1.125rem',
    xl: '1.25rem',
    '2xl': '1.5rem',
    '3xl': '1.875rem',
    '4xl': '2.25rem',
  },
  fontWeight: {
    normal: '400',
    medium: '500',
    semibold: '600',
    bold: '700',
  },
  transitions: {
    fast: '150ms ease-in-out',
    normal: '250ms ease-in-out',
    slow: '350ms ease-in-out',
  },
  shadows: {
    sm: '0 1px 2px 0 rgb(0 0 0 / 0.05)',
    default: '0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)',
    md: '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
    lg: '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
    xl: '0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)',
  },
  zIndex: {
    dropdown: '1000',
    sticky: '1020',
    fixed: '1030',
    modalBackdrop: '1040',
    modal: '1050',
    popover: '1060',
    tooltip: '1070',
    toast: '1080',
  },
};

// Accessibility-compliant color palettes
export const accessibilityColors = {
  light: {
    primary: 'rgb(37 99 235)', // WCAG AA compliant
    text: 'rgb(17 24 39)',
    background: 'rgb(255 255 255)',
  },
  dark: {
    primary: 'rgb(96 165 250)', // WCAG AA compliant
    text: 'rgb(248 250 252)',
    background: 'rgb(3 7 18)',
  },
};

// High contrast theme for accessibility
export const highContrastTheme: Partial<ThemeConfig> = {
  colors: {
    ...defaultThemeConfig.colors,
    text: {
      primary: 'rgb(0 0 0)',
      secondary: 'rgb(0 0 0)',
      tertiary: 'rgb(64 64 64)',
      inverse: 'rgb(255 255 255)',
    },
    border: {
      default: 'rgb(0 0 0)',
      secondary: 'rgb(64 64 64)',
      accent: 'rgb(37 99 235)',
    },
  },
};

// Utility function to generate CSS custom properties
export function generateCSSCustomProperties(config: ThemeConfig): string {
  const properties: string[] = [];

  // Color properties
  Object.entries(config.colors).forEach(([colorName, colorValue]) => {
    if (typeof colorValue === 'object' && colorValue !== null) {
      Object.entries(colorValue).forEach(([shade, value]) => {
        properties.push(`--color-${colorName}-${shade}: ${value.replace('rgb(', '').replace(')', '')};`);
      });
    }
  });

  // Other properties
  Object.entries(config.spacing).forEach(([key, value]) => {
    properties.push(`--spacing-${key}: ${value};`);
  });

  Object.entries(config.borderRadius).forEach(([key, value]) => {
    properties.push(`--radius${key === 'default' ? '' : `-${key}`}: ${value};`);
  });

  Object.entries(config.fontSize).forEach(([key, value]) => {
    properties.push(`--font-size-${key}: ${value};`);
  });

  Object.entries(config.fontWeight).forEach(([key, value]) => {
    properties.push(`--font-weight-${key}: ${value};`);
  });

  Object.entries(config.transitions).forEach(([key, value]) => {
    properties.push(`--transition-${key}: ${value};`);
  });

  Object.entries(config.shadows).forEach(([key, value]) => {
    properties.push(`--shadow${key === 'default' ? '' : `-${key}`}: ${value};`);
  });

  Object.entries(config.zIndex).forEach(([key, value]) => {
    properties.push(`--z-${key.replace(/([A-Z])/g, '-$1').toLowerCase()}: ${value};`);
  });

  return `:root {\n  ${properties.join('\n  ')}\n}`;
}