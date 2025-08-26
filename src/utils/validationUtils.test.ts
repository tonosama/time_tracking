import { describe, it, expect } from 'vitest';
import { Project } from '../types';
import { 
  validateProjectName, 
  checkProjectNameDuplicate,
  validateProjectForm,
  ValidationResult 
} from './validationUtils';

describe('Project Validation Utils', () => {
  const mockProjects: Project[] = [
    {
      id: 1,
      name: 'Web Development',
      status: 'active',
      effective_at: '2024-01-01T00:00:00Z',
      color: '#28a745',
    },
    {
      id: 2,
      name: 'API Project',
      status: 'active',
      effective_at: '2024-01-01T00:00:00Z',
      color: '#dc3545',
    },
    {
      id: 3,
      name: 'Documentation',
      status: 'active',
      effective_at: '2024-01-01T00:00:00Z',
      color: '#ffc107',
    }
  ];

  describe('validateProjectName', () => {
    it('有効なプロジェクト名を検証する', () => {
      const result = validateProjectName('Valid Project Name');
      expect(result.isValid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('空のプロジェクト名を検証する', () => {
      const result = validateProjectName('');
      expect(result.isValid).toBe(false);
      expect(result.errors).toContain('プロジェクト名は必須です');
    });

    it('空白のみのプロジェクト名を検証する', () => {
      const result = validateProjectName('   ');
      expect(result.isValid).toBe(false);
      expect(result.errors).toContain('プロジェクト名は必須です');
    });

    it('短すぎるプロジェクト名を検証する', () => {
      const result = validateProjectName('ab');
      expect(result.isValid).toBe(false);
      expect(result.errors).toContain('プロジェクト名は3文字以上で入力してください');
    });

    it('長すぎるプロジェクト名を検証する', () => {
      const longName = 'a'.repeat(101);
      const result = validateProjectName(longName);
      expect(result.isValid).toBe(false);
      expect(result.errors).toContain('プロジェクト名は100文字以下で入力してください');
    });

    it('特殊文字を含むプロジェクト名を検証する', () => {
      const result = validateProjectName('Project<>Name');
      expect(result.isValid).toBe(false);
      expect(result.errors).toContain('プロジェクト名に使用できない文字が含まれています');
    });

    it('複数のエラーがある場合を検証する', () => {
      const result = validateProjectName('<>');
      expect(result.isValid).toBe(false);
      expect(result.errors).toContain('プロジェクト名は3文字以上で入力してください');
      expect(result.errors).toContain('プロジェクト名に使用できない文字が含まれています');
    });
  });

  describe('checkProjectNameDuplicate', () => {
    it('重複していないプロジェクト名を検証する', () => {
      const result = checkProjectNameDuplicate('New Project', mockProjects, null);
      expect(result.isValid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('既存プロジェクトと重複する名前を検証する', () => {
      const result = checkProjectNameDuplicate('Web Development', mockProjects, null);
      expect(result.isValid).toBe(false);
      expect(result.errors).toContain('このプロジェクト名は既に使用されています');
    });

    it('大文字小文字を区別しない重複チェック', () => {
      const result = checkProjectNameDuplicate('web development', mockProjects, null);
      expect(result.isValid).toBe(false);
      expect(result.errors).toContain('このプロジェクト名は既に使用されています');
    });

    it('編集時に自分自身との重複を許可する', () => {
      const result = checkProjectNameDuplicate('Web Development', mockProjects, 1);
      expect(result.isValid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('編集時に他のプロジェクトとの重複を検出する', () => {
      const result = checkProjectNameDuplicate('API Project', mockProjects, 1);
      expect(result.isValid).toBe(false);
      expect(result.errors).toContain('このプロジェクト名は既に使用されています');
    });

    it('前後の空白を除去して重複チェックする', () => {
      const result = checkProjectNameDuplicate('  Web Development  ', mockProjects, null);
      expect(result.isValid).toBe(false);
      expect(result.errors).toContain('このプロジェクト名は既に使用されています');
    });
  });

  describe('validateProjectForm', () => {
    it('有効なプロジェクトフォームを検証する', () => {
      const formData = {
        name: 'New Project',
        color: '#28a745',
        description: 'Project description'
      };
      const result = validateProjectForm(formData, mockProjects, null);
      expect(result.isValid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('無効なプロジェクトフォームを検証する', () => {
      const formData = {
        name: '',
        color: '#28a745',
        description: 'Project description'
      };
      const result = validateProjectForm(formData, mockProjects, null);
      expect(result.isValid).toBe(false);
      expect(result.errors).toContain('プロジェクト名は必須です');
    });

    it('重複する名前のフォームを検証する', () => {
      const formData = {
        name: 'Web Development',
        color: '#28a745',
        description: 'Project description'
      };
      const result = validateProjectForm(formData, mockProjects, null);
      expect(result.isValid).toBe(false);
      expect(result.errors).toContain('このプロジェクト名は既に使用されています');
    });

    it('複数のエラーがあるフォームを検証する', () => {
      const formData = {
        name: 'ab',
        color: '#28a745',
        description: 'Project description'
      };
      const result = validateProjectForm(formData, mockProjects, null);
      expect(result.isValid).toBe(false);
      expect(result.errors).toContain('プロジェクト名は3文字以上で入力してください');
    });

    it('編集時に自分自身との重複を許可する', () => {
      const formData = {
        name: 'Web Development',
        color: '#28a745',
        description: 'Updated description'
      };
      const result = validateProjectForm(formData, mockProjects, 1);
      expect(result.isValid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });
  });

  describe('ValidationResult', () => {
    it('ValidationResultの構造を検証する', () => {
      const result: ValidationResult = {
        isValid: true,
        errors: []
      };
      expect(result.isValid).toBe(true);
      expect(Array.isArray(result.errors)).toBe(true);
    });

    it('エラーがある場合のValidationResultを検証する', () => {
      const result: ValidationResult = {
        isValid: false,
        errors: ['エラー1', 'エラー2']
      };
      expect(result.isValid).toBe(false);
      expect(result.errors).toHaveLength(2);
      expect(result.errors).toContain('エラー1');
      expect(result.errors).toContain('エラー2');
    });
  });
});
