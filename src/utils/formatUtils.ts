/**
 * ミリ秒を人間が読みやすい形式に変換
 */
export function formatDuration(milliseconds: number): string {
  const seconds = Math.floor(milliseconds / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  
  if (hours > 0) {
    const remainingMinutes = minutes % 60
    const remainingSeconds = seconds % 60
    return `${hours}:${remainingMinutes.toString().padStart(2, '0')}:${remainingSeconds.toString().padStart(2, '0')}`
  } else if (minutes > 0) {
    const remainingSeconds = seconds % 60
    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`
  } else {
    return `0:${seconds.toString().padStart(2, '0')}`
  }
}

/**
 * Dateを時刻文字列に変換 (HH:MM)
 */
export function formatTime(date: Date): string {
  return date.toLocaleTimeString('ja-JP', { 
    hour: '2-digit', 
    minute: '2-digit',
    hour12: false 
  })
}

/**
 * 短い時間形式 (例: 2h 30m)
 */
export function formatDurationShort(milliseconds: number): string {
  const minutes = Math.floor(milliseconds / (1000 * 60))
  const hours = Math.floor(minutes / 60)
  const remainingMinutes = minutes % 60
  
  if (hours > 0) {
    if (remainingMinutes > 0) {
      return `${hours}h ${remainingMinutes}m`
    } else {
      return `${hours}h`
    }
  } else if (minutes > 0) {
    return `${minutes}m`
  } else {
    return '0m'
  }
}

