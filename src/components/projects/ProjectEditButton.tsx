import React from 'react';

interface ProjectEditButtonProps {
  onClick: () => void;
  disabled?: boolean;
  className?: string;
  'aria-label'?: string;
}

/**
 * プロジェクト編集ボタンコンポーネント
 * 
 * @param onClick - クリック時のコールバック関数
 * @param disabled - 無効化状態（デフォルト: false）
 * @param className - カスタムCSSクラス名
 * @param aria-label - アクセシビリティ用ラベル（デフォルト: "Edit project"）
 */
export const ProjectEditButton: React.FC<ProjectEditButtonProps> = ({
  onClick,
  disabled = false,
  className = '',
  'aria-label': ariaLabel = 'Edit project'
}) => {
  const handleClick = (e: React.MouseEvent) => {
    console.log('ProjectEditButton clicked', { disabled, ariaLabel });
    e.preventDefault();
    e.stopPropagation();
    e.nativeEvent.stopImmediatePropagation();
    if (!disabled) {
      console.log('ProjectEditButton calling onClick');
      onClick();
    } else {
      console.log('ProjectEditButton is disabled');
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      if (!disabled) {
        onClick();
      }
    }
  };

  return (
    <button
      type="button"
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      disabled={disabled}
      className={`project-edit-button ${className}`.trim()}
      aria-label={ariaLabel}
      title={ariaLabel}
    >
      <span className="edit-icon" aria-hidden="true">
        ✏️
      </span>
    </button>
  );
};

export default ProjectEditButton;
