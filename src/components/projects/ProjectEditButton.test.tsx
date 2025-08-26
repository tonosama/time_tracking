import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { ProjectEditButton } from './ProjectEditButton';

describe('ProjectEditButton', () => {
  const mockOnClick = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('プロジェクト編集ボタンが正しく表示される', () => {
    render(<ProjectEditButton onClick={mockOnClick} />);
    
    const button = screen.getByRole('button', { name: /edit project/i });
    expect(button).toBeInTheDocument();
    expect(button).toHaveTextContent('✏️');
  });

  it('クリックイベントが正しく動作する', () => {
    render(<ProjectEditButton onClick={mockOnClick} />);
    
    const button = screen.getByRole('button', { name: /edit project/i });
    fireEvent.click(button);
    
    expect(mockOnClick).toHaveBeenCalledTimes(1);
  });

  it('無効化状態ではクリックイベントが動作しない', () => {
    render(<ProjectEditButton onClick={mockOnClick} disabled={true} />);
    
    const button = screen.getByRole('button', { name: /edit project/i });
    expect(button).toBeDisabled();
    
    fireEvent.click(button);
    expect(mockOnClick).not.toHaveBeenCalled();
  });

  it('カスタムクラス名が適用される', () => {
    const customClass = 'custom-edit-button';
    render(<ProjectEditButton onClick={mockOnClick} className={customClass} />);
    
    const button = screen.getByRole('button', { name: /edit project/i });
    expect(button).toHaveClass(customClass);
  });

  it('カスタムaria-labelが適用される', () => {
    const customLabel = 'カスタム編集ラベル';
    render(<ProjectEditButton onClick={mockOnClick} aria-label={customLabel} />);
    
    const button = screen.getByRole('button', { name: customLabel });
    expect(button).toBeInTheDocument();
  });

  it('デフォルトのaria-labelが適用される', () => {
    render(<ProjectEditButton onClick={mockOnClick} />);
    
    const button = screen.getByRole('button', { name: /edit project/i });
    expect(button).toBeInTheDocument();
  });

  it('ボタンがフォーカス可能である', () => {
    render(<ProjectEditButton onClick={mockOnClick} />);
    
    const button = screen.getByRole('button', { name: /edit project/i });
    button.focus();
    
    expect(button).toHaveFocus();
  });

  it('Enterキーでクリックイベントが発火する', () => {
    render(<ProjectEditButton onClick={mockOnClick} />);
    
    const button = screen.getByRole('button', { name: /edit project/i });
    button.focus();
    fireEvent.keyDown(button, { key: 'Enter', code: 'Enter' });
    
    expect(mockOnClick).toHaveBeenCalledTimes(1);
  });

  it('Spaceキーでクリックイベントが発火する', () => {
    render(<ProjectEditButton onClick={mockOnClick} />);
    
    const button = screen.getByRole('button', { name: /edit project/i });
    button.focus();
    fireEvent.keyDown(button, { key: ' ', code: 'Space' });
    
    expect(mockOnClick).toHaveBeenCalledTimes(1);
  });
});
