interface LoadingProps {
  message?: string
  size?: 'sm' | 'md' | 'lg'
}

export function Loading({ message = '読み込み中...', size = 'md' }: LoadingProps) {
  return (
    <div className={`loading loading-${size}`} role="status">
      <div className="spinner" aria-hidden="true" />
      <span className="loading-text">{message}</span>
    </div>
  )
}
