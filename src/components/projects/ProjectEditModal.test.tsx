import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import ProjectEditModal from './ProjectEditModal';

describe('ProjectEditModal', () => {
  const mockProject = {
    id: 1,
    name: 'Web Development',
    color: '#3b82f6',
    status: 'active' as const,
    effective_at: new Date().toISOString(),
    // description と totalTime は削除（Project型に存在しない）
  };

  const mockOnSave = vi.fn();
  const mockOnCancel = vi.fn();
  const mockOnClose = vi.fn();

  const defaultProps = {
    project: mockProject,
    isOpen: true,
    onSave: mockOnSave,
    onCancel: mockOnCancel,
    onClose: mockOnClose
  };

  it('プロジェクト編集モーダルが正しく表示される', () => {
    render(<ProjectEditModal {...defaultProps} />);
    
    expect(screen.getByText('Edit Project')).toBeInTheDocument();
    expect(screen.getByDisplayValue('Web Development')).toBeInTheDocument();
    expect(screen.getByDisplayValue('Frontend and UI development tasks')).toBeInTheDocument();
  });

  it('モーダルが閉じている時は表示されない', () => {
    render(<ProjectEditModal {...defaultProps} isOpen={false} />);
    
    expect(screen.queryByText('Edit Project')).not.toBeInTheDocument();
  });

  it('閉じるボタンがクリックされるとonCloseが呼ばれる', () => {
    render(<ProjectEditModal {...defaultProps} />);
    
    const closeButton = screen.getByRole('button', { name: /close/i });
    fireEvent.click(closeButton);
    
    expect(mockOnClose).toHaveBeenCalledTimes(1);
  });

  it('キャンセルボタンがクリックされるとonCancelが呼ばれる', () => {
    render(<ProjectEditModal {...defaultProps} />);
    
    const cancelButton = screen.getByRole('button', { name: /cancel/i });
    fireEvent.click(cancelButton);
    
    expect(mockOnCancel).toHaveBeenCalledTimes(1);
  });

  it('保存ボタンがクリックされるとonSaveが呼ばれる', () => {
    render(<ProjectEditModal {...defaultProps} />);
    
    const saveButton = screen.getByRole('button', { name: /save changes/i });
    fireEvent.click(saveButton);
    
    expect(mockOnSave).toHaveBeenCalledWith(mockProject);
  });

  it('プロジェクト名が入力フィールドに表示される', () => {
    render(<ProjectEditModal {...defaultProps} />);
    
    const nameInput = screen.getByLabelText(/project name/i);
    expect(nameInput).toHaveValue('Web Development');
  });

  it('プロジェクト説明が入力フィールドに表示される', () => {
    render(<ProjectEditModal {...defaultProps} />);
    
    const descriptionInput = screen.getByLabelText(/description/i);
    expect(descriptionInput).toHaveValue('Frontend and UI development tasks');
  });

  it('色選択オプションが表示される', () => {
    render(<ProjectEditModal {...defaultProps} />);
    
    expect(screen.getByText('Project Color')).toBeInTheDocument();
    // 色選択肢が表示されることを確認
    const colorOptions = screen.getAllByRole('button', { name: /color/i });
    expect(colorOptions).toHaveLength(5); // 5つの色オプション
  });
});

