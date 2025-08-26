import { TimeEntry } from '@/types'

/**
 * 時間エントリーを日付でグループ化
 */
export function groupEntriesByDate(entries: TimeEntry[]): Record<string, TimeEntry[]> {
  return entries.reduce((groups, entry) => {
    const date = new Date(entry.start_time).toDateString()
    if (!groups[date]) {
      groups[date] = []
    }
    groups[date].push(entry)
    return groups
  }, {} as Record<string, TimeEntry[]>)
}

/**
 * 日付を相対的な表現に変換 (Today, Yesterday, etc.)
 */
export function formatRelativeDate(date: Date): string {
  const today = new Date()
  const yesterday = new Date(today)
  yesterday.setDate(yesterday.getDate() - 1)
  
  const dateString = date.toDateString()
  const todayString = today.toDateString()
  const yesterdayString = yesterday.toDateString()
  
  if (dateString === todayString) {
    return 'Today'
  } else if (dateString === yesterdayString) {
    return 'Yesterday'
  } else {
    return date.toLocaleDateString('ja-JP', { 
      month: 'short', 
      day: 'numeric',
      weekday: 'short'
    })
  }
}

/**
 * 日付をフォーマットして表示
 */
export function formatDate(dateString: string): string {
  const date = new Date(dateString)
  return date.toLocaleDateString('ja-JP', {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  })
}

