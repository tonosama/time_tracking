import { InputHTMLAttributes, forwardRef } from 'react'

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string
  error?: string
  helpText?: string
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ label, error, helpText, className = '', id, ...props }, ref) => {
    const inputId = id || `input-${Math.random().toString(36).substr(2, 9)}`
    const errorId = error ? `${inputId}-error` : undefined
    const helpId = helpText ? `${inputId}-help` : undefined

    return (
      <div className="input-group">
        {label && (
          <label htmlFor={inputId} className="input-label">
            {label}
          </label>
        )}
        <input
          ref={ref}
          id={inputId}
          className={`input ${error ? 'input-error' : ''} ${className}`}
          aria-invalid={error ? 'true' : 'false'}
          aria-describedby={[errorId, helpId].filter(Boolean).join(' ') || undefined}
          {...props}
        />
        {error && (
          <span id={errorId} className="input-error-text" role="alert">
            {error}
          </span>
        )}
        {helpText && !error && (
          <span id={helpId} className="input-help-text">
            {helpText}
          </span>
        )}
      </div>
    )
  }
)

Input.displayName = 'Input'
