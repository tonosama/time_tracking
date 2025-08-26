import { Project } from '../types';

/**
 * バリデーション結果の型定義
 */
export interface ValidationResult {
  isValid: boolean;
  errors: string[];
}

/**
 * プロジェクトフォームデータの型定義
 */
export interface ProjectFormData {
  name: string;
  color: string;
  description?: string;
}

/**
 * プロジェクト名の基本バリデーション
 * 
 * @param name - 検証するプロジェクト名
 * @returns バリデーション結果
 */
export function validateProjectName(name: string): ValidationResult {
  const errors: string[] = [];
  const trimmedName = name.trim();

  // 必須チェック
  if (!trimmedName) {
    errors.push('プロジェクト名は必須です');
    return { isValid: false, errors };
  }

  // 長さチェック（3-100文字）
  if (trimmedName.length < 3) {
    errors.push('プロジェクト名は3文字以上で入力してください');
  }

  if (trimmedName.length > 100) {
    errors.push('プロジェクト名は100文字以下で入力してください');
  }

  // 特殊文字チェック
  const invalidChars = /[<>:"\\|?*]/;
  if (invalidChars.test(trimmedName)) {
    errors.push('プロジェクト名に使用できない文字が含まれています');
  }

  return {
    isValid: errors.length === 0,
    errors
  };
}

/**
 * プロジェクト名の重複チェック
 * 
 * @param name - 検証するプロジェクト名
 * @param projects - 既存のプロジェクトリスト
 * @param currentProjectId - 現在編集中のプロジェクトID（編集時のみ）
 * @returns バリデーション結果
 */
export function checkProjectNameDuplicate(
  name: string, 
  projects: Project[], 
  currentProjectId: number | null
): ValidationResult {
  const trimmedName = name.trim().toLowerCase();
  const errors: string[] = [];

  // 既存プロジェクトとの重複チェック
  const duplicateProject = projects.find(project => {
    const projectName = project.name.toLowerCase();
    // 編集時は自分自身との重複を許可
    if (currentProjectId && project.id === currentProjectId) {
      return false;
    }
    return projectName === trimmedName;
  });

  if (duplicateProject) {
    errors.push('このプロジェクト名は既に使用されています');
  }

  return {
    isValid: errors.length === 0,
    errors
  };
}

/**
 * プロジェクトフォーム全体のバリデーション
 * 
 * @param formData - フォームデータ
 * @param projects - 既存のプロジェクトリスト
 * @param currentProjectId - 現在編集中のプロジェクトID（編集時のみ）
 * @returns バリデーション結果
 */
export function validateProjectForm(
  formData: ProjectFormData,
  projects: Project[],
  currentProjectId: number | null
): ValidationResult {
  const errors: string[] = [];

  // プロジェクト名の基本バリデーション
  const nameValidation = validateProjectName(formData.name);
  errors.push(...nameValidation.errors);

  // プロジェクト名の重複チェック
  const duplicateValidation = checkProjectNameDuplicate(
    formData.name, 
    projects, 
    currentProjectId
  );
  errors.push(...duplicateValidation.errors);

  // 色のバリデーション（基本的な形式チェック）
  if (!formData.color || !formData.color.match(/^#[0-9A-Fa-f]{6}$/)) {
    errors.push('有効な色を選択してください');
  }

  return {
    isValid: errors.length === 0,
    errors
  };
}

/**
 * リアルタイムバリデーション用の軽量チェック
 * 
 * @param name - 検証するプロジェクト名
 * @returns 基本バリデーション結果（重複チェックは含まない）
 */
export function validateProjectNameRealtime(name: string): ValidationResult {
  return validateProjectName(name);
}

/**
 * エラーメッセージを日本語で取得
 * 
 * @param errorCode - エラーコード
 * @returns エラーメッセージ
 */
export function getValidationErrorMessage(errorCode: string): string {
  const errorMessages: Record<string, string> = {
    'REQUIRED': 'プロジェクト名は必須です',
    'MIN_LENGTH': 'プロジェクト名は3文字以上で入力してください',
    'MAX_LENGTH': 'プロジェクト名は100文字以下で入力してください',
    'INVALID_CHARS': 'プロジェクト名に使用できない文字が含まれています',
    'DUPLICATE': 'このプロジェクト名は既に使用されています',
    'INVALID_COLOR': '有効な色を選択してください'
  };

  return errorMessages[errorCode] || '入力内容に問題があります';
}