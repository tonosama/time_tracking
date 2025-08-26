#!/bin/bash

# フロントエンド・バックエンド統合テスト実行スクリプト
# 
# このスクリプトは以下の手順で統合テストを実行します：
# 1. Tauriアプリケーションを起動
# 2. フロントエンド・バックエンド統合テストを実行
# 3. アプリケーションを停止

set -e

echo "🚀 フロントエンド・バックエンド統合テストを開始します..."

# 既存のプロセスを停止
echo "📋 既存のプロセスを停止中..."
pkill -f "tauri|cargo.*tauri|vite|time-tracker-go" || true

# 少し待機
sleep 2

# Tauriアプリケーションをバックグラウンドで起動
echo "🔧 Tauriアプリケーションを起動中..."
npm run tauri:dev &
TAURI_PID=$!

# アプリケーションの起動を待機
echo "⏳ アプリケーションの起動を待機中..."
sleep 10

# 統合テストを実行
echo "🧪 統合テストを実行中..."
npm test -- tests/integration/frontend_backend_integration_test.ts

# テスト結果を保存
TEST_EXIT_CODE=$?

# Tauriアプリケーションを停止
echo "🛑 Tauriアプリケーションを停止中..."
kill $TAURI_PID || true

# 少し待機
sleep 2

# 結果を表示
if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo "✅ 統合テストが正常に完了しました！"
else
    echo "❌ 統合テストが失敗しました。"
    exit $TEST_EXIT_CODE
fi
