import { Component, ErrorInfo, ReactNode } from 'react'

interface Props {
  children: ReactNode
  fallback?: ReactNode
}

interface State {
  hasError: boolean
  error?: Error
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = { hasError: false }
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('ErrorBoundary caught an error:', error, errorInfo)
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback
      }

      return (
        <div className="error-boundary">
          <h2>エラーが発生しました</h2>
          <p>申し訳ございませんが、予期しないエラーが発生しました。</p>
          <details>
            <summary>エラー詳細</summary>
            <pre>{this.state.error?.stack}</pre>
          </details>
          <button 
            onClick={() => this.setState({ hasError: false, error: undefined })}
            className="btn btn-primary"
          >
            再試行
          </button>
        </div>
      )
    }

    return this.props.children
  }
}
