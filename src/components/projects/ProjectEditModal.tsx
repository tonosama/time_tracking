import React, { useState, useEffect } from 'react';
import { Button } from '../common/Button';
import { Input } from '../common/Input';
import { ColorPicker } from './ColorPicker';
import { Project } from '../../types';
import { validateProjectForm } from '../../utils/validationUtils';

interface ProjectEditModalProps {
  project: Project;
  isOpen: boolean;
  onSave: (project: Project) => void;
  onCancel: () => void;
  onClose: () => void;
  projects?: Project[]; // 重複チェック用のプロジェクトリスト
}

export const ProjectEditModal: React.FC<ProjectEditModalProps> = ({
  project,
  isOpen,
  onSave,
  onCancel,
  onClose,
  projects = []
}) => {
  const [formData, setFormData] = useState<Project>(project);
  const [validationErrors, setValidationErrors] = useState<string[]>([]);
  const [isValidating, setIsValidating] = useState(false);

  // プロジェクトが変更された時にフォームデータを更新
  useEffect(() => {
    setFormData(project);
  }, [project]);

  const handleInputChange = (field: keyof Project, value: string) => {
    setFormData(prev => ({
      ...prev,
      [field]: value
    }));

    // プロジェクト名が変更された場合、リアルタイムバリデーションを実行
    if (field === 'name') {
      validateFormData({
        ...formData,
        [field]: value
      });
    }
  };

  const validateFormData = (data: Project) => {
    const formData = {
      name: data.name,
      color: data.color || '#007bff',
      description: (data as any).description || ''
    };
    const validation = validateProjectForm(formData, projects, project.id);
    setValidationErrors(validation.errors);
    return validation.isValid;
  };

  const handleSave = async () => {
    setIsValidating(true);
    
    // フォーム全体のバリデーションを実行
    const isValid = validateFormData(formData);
    
    if (isValid) {
      onSave(formData);
    } else {
      setIsValidating(false);
    }
  };

  const handleCancel = () => {
    setFormData(project); // フォームデータをリセット
    setValidationErrors([]); // バリデーションエラーをクリア
    setIsValidating(false);
    onCancel();
  };

  const handleClose = () => {
    setFormData(project); // フォームデータをリセット
    setValidationErrors([]); // バリデーションエラーをクリア
    setIsValidating(false);
    onClose();
  };

  if (!isOpen) {
    return null;
  }

  return (
    <div className="modal-backdrop" onClick={handleClose}>
      <div className="project-edit-modal" onClick={(e) => e.stopPropagation()}>
        {/* モーダルヘッダー */}
        <div className="modal-header">
          <h2>Edit Project</h2>
          <button
            className="close-button"
            onClick={handleClose}
            aria-label="Close"
          >
            ✕
          </button>
        </div>

        {/* 編集フォーム */}
        <div className="modal-content">
          {/* プロジェクト名 */}
          <div className="form-group">
            <label htmlFor="project-name">Project Name</label>
            <Input
              id="project-name"
              type="text"
              value={formData.name}
              onChange={(e) => handleInputChange('name', e.target.value)}
              placeholder="Enter project name"
              error={validationErrors.length > 0 ? validationErrors[0] : undefined}
            />
            {validationErrors.length > 1 && (
              <div className="validation-errors">
                {validationErrors.slice(1).map((error, index) => (
                  <div key={index} className="error-message">
                    {error}
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* プロジェクト色 */}
          <div className="form-group">
            <label>Project Color</label>
            <ColorPicker
              selectedColor={formData.color || '#007bff'}
              onColorChange={(color) => handleInputChange('color', color)}
            />
          </div>

          {/* プロジェクト説明 */}
          <div className="form-group">
            <label htmlFor="project-description">Description (Optional)</label>
            <textarea
              id="project-description"
              value={(formData as any).description || ''}
              onChange={(e) => handleInputChange('description' as keyof Project, e.target.value)}
              placeholder="Enter project description"
              rows={3}
            />
          </div>
        </div>

        {/* アクションボタン */}
        <div className="modal-actions">
          <Button
            variant="secondary"
            onClick={handleCancel}
          >
            Cancel
          </Button>
          <Button
            variant="primary"
            onClick={handleSave}
            disabled={validationErrors.length > 0 || isValidating}
            loading={isValidating}
          >
            Save Changes
          </Button>
        </div>
      </div>
    </div>
  );
};

export default ProjectEditModal;

