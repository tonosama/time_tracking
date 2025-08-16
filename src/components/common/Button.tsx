import { ReactNode, ButtonHTMLAttributes } from 'react'

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'success' | 'warning' | 'danger'
  size?: 'sm' | 'md' | 'lg'
  loading?: boolean
  children: ReactNode
}

export function Button({ 
  variant = 'primary', 
  size = 'md', 
  loading = false, 
  children, 
  className = '', 
  disabled,
  ...props 
}: ButtonProps) {
  const baseClass = 'btn'
  const variantClass = `btn-${variant}`
  const sizeClass = size !== 'md' ? `btn-${size}` : ''
  const loadingClass = loading ? 'btn-loading' : ''
  
  const classes = [baseClass, variantClass, sizeClass, loadingClass, className]
    .filter(Boolean)
    .join(' ')

  return (
    <button
      className={classes}
      disabled={disabled || loading}
      {...props}
    >
      {loading && <span className="spinner" aria-hidden="true" />}
      {children}
    </button>
  )
}
