import React from 'react';

interface ColorPickerProps {
  selectedColor: string;
  onColorChange: (color: string) => void;
}

const PROJECT_COLORS = [
  { value: '#28a745', label: 'Green' },
  { value: '#dc3545', label: 'Red' },
  { value: '#ffc107', label: 'Yellow' },
  { value: '#007bff', label: 'Blue' },
  { value: '#6f42c1', label: 'Purple' }
];

export const ColorPicker: React.FC<ColorPickerProps> = ({
  selectedColor,
  onColorChange
}) => {
  return (
    <div className="color-picker">
      <div className="color-options">
        {PROJECT_COLORS.map((color) => (
          <button
            key={color.value}
            className={`color-option ${selectedColor === color.value ? 'selected' : ''}`}
            style={{ backgroundColor: color.value }}
            onClick={() => onColorChange(color.value)}
            aria-label={`Select ${color.label} color`}
            title={color.label}
          />
        ))}
      </div>
    </div>
  );
};

