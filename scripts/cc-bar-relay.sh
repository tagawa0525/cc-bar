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

# session_id をファイル名として安全な文字種に正規化
safe_session_id=$(printf '%s' "$session_id" | tr -c 'A-Za-z0-9_.-' '_')

if [ -z "$safe_session_id" ]; then
    echo "Error: session_id is invalid after sanitization" >&2
    exit 1
fi

# セッションファイルパス
session_file="${SESSIONS_DIR}/${safe_session_id}.json"
temp_file="${session_file}.tmp.$$"

# 既存セッションファイルの subagent_completed_count を引き継ぐ
existing_subagent_count=""
if [ -f "$session_file" ]; then
    # 既存ファイルからカウントを取得（存在しない場合は空文字）
    existing_subagent_count=$(jq -r '.subagent_completed_count // empty' "$session_file" 2>/dev/null || echo "")
fi

# 新しいJSONに subagent_completed_count が含まれているか確認
has_new_subagent_count=$(printf '%s' "$json_input" | jq -r 'has("subagent_completed_count")' 2>/dev/null || echo "false")

# 書き込むJSONを決定
json_to_write="$json_input"
if [ "$has_new_subagent_count" != "true" ] && [ -n "${existing_subagent_count:-}" ]; then
    # 新しいJSONにフィールドがなく、既存ファイルにカウントがある場合は引き継ぐ
    json_to_write=$(printf '%s' "$json_input" | jq --argjson cnt "$existing_subagent_count" '. + {subagent_completed_count: $cnt}' 2>/dev/null || echo "$json_input")
fi

# Atomic write: 一時ファイルに書き込んでからリネーム
if printf '%s' "$json_to_write" | jq . > "$temp_file" 2>/dev/null; then
    mv "$temp_file" "$session_file"

    # Claude Code側に短い状態を返す
    context_used=$(printf '%s' "$json_input" | jq -r '.context_window.used_percentage // "?"' 2>/dev/null || echo "?")
    model=$(printf '%s' "$json_input" | jq -r '.model.display_name // "?"' 2>/dev/null || echo "?")

    echo "[$model ${context_used}%]"
else
    rm -f "$temp_file"
    echo "Error: Failed to write session file" >&2
    exit 1
fi
