// アプリケーション定数

/**
 * 時間関連の定数
 */
export const TIME_CONSTANTS = {
  MILLISECONDS_IN_SECOND: 1000,
  SECONDS_IN_MINUTE: 60,
  MINUTES_IN_HOUR: 60,
  HOURS_IN_DAY: 24,
} as const;

/**
 * UI関連の定数
 */
export const UI_CONSTANTS = {
  DEBOUNCE_DELAY: 300,
  ANIMATION_DURATION: 200,
  TOAST_DURATION: 3000,
} as const;

/**
 * API関連の定数
 */
export const API_CONSTANTS = {
  REQUEST_TIMEOUT: 5000,
  RETRY_COUNT: 3,
  RETRY_DELAY: 1000,
} as const;

/**
 * ストレージキー
 */
export const STORAGE_KEYS = {
  THEME: 'time-tracker-theme',
  LANGUAGE: 'time-tracker-language',
  USER_PREFERENCES: 'time-tracker-preferences',
} as const;

/**
 * デフォルトの色パレット
 */
export const DEFAULT_COLORS = [
  '#3b82f6', // blue
  '#ef4444', // red
  '#10b981', // emerald
  '#f59e0b', // amber
  '#8b5cf6', // violet
  '#ec4899', // pink
  '#06b6d4', // cyan
  '#84cc16', // lime
] as const;