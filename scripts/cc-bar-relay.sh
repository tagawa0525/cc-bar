#!/usr/bin/env bash
# cc-bar-relay.sh - Claude Code Status Line Relay
# Status LineからのJSONをcc-barに中継するスクリプト
# 用法: ~/.claude/settings.json の statusLine に設定

set -euo pipefail

# XDG_RUNTIME_DIRの取得
RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
SESSIONS_DIR="${RUNTIME_DIR}/cc-bar/sessions"

# ディレクトリを作成
mkdir -p "$SESSIONS_DIR"

# stdin からJSONを読み込む（複数行対応）
json_input=$(cat)

# session_idを抽出
session_id=$(echo "$json_input" | jq -r '.session_id // empty' 2>/dev/null || echo "")

if [ -z "$session_id" ]; then
    # session_idが取得できない場合はエラーを返して終了
    echo "Error: Could not extract session_id from input" >&2
    exit 1
fi

# セッションファイルパス
session_file="${SESSIONS_DIR}/${session_id}.json"
temp_file="${session_file}.tmp.$$"

# Atomic write: 一時ファイルに書き込んでからリネーム
if echo "$json_input" | jq . > "$temp_file" 2>/dev/null; then
    mv "$temp_file" "$session_file"

    # Claude Code側に短い状態を返す
    context_used=$(echo "$json_input" | jq -r '.context_window.used_percentage // "?"' 2>/dev/null || echo "?")
    model=$(echo "$json_input" | jq -r '.model.display_name // "?"' 2>/dev/null || echo "?")

    echo "[$model ${context_used}%]"
else
    rm -f "$temp_file"
    echo "Error: Failed to write session file" >&2
    exit 1
fi
