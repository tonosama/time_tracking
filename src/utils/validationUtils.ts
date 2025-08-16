// バリデーション関数 - 入力値の検証

/**
 * プロジェクト名のバリデーション
 */
export const validateProjectName = (name: string): boolean => {
  return name.trim().length > 0 && name.trim().length <= 100;
};

/**
 * タスク名のバリデーション
 */
export const validateTaskName = (name: string): boolean => {
  return name.trim().length > 0 && name.trim().length <= 200;
};

/**
 * メールアドレスのバリデーション
 */
export const validateEmail = (email: string): boolean => {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
};

/**
 * 時間形式のバリデーション (HH:mm)
 */
export const validateTimeFormat = (time: string): boolean => {
  const timeRegex = /^([0-1]?[0-9]|2[0-3]):[0-5][0-9]$/;
  return timeRegex.test(time);
};