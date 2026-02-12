#!/usr/bin/env bash
# cc-bar-subagent-hook.sh - Claude Code SubagentStop Hook
# SubagentStop フックでサブエージェント完了をトラッキング
# 用法: ~/.claude/settings.json の hooks.SubagentStop に設定

set -euo pipefail

# XDG_RUNTIME_DIRの取得
RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
SUBAGENTS_DIR="${RUNTIME_DIR}/cc-bar/subagents"

# ディレクトリを作成
mkdir -p "$SUBAGENTS_DIR"

# 入力JSON: {"session_id": "...", "subagent_id": "...", "timestamp": ...}
json_input=$(cat)

# session_idを抽出
session_id=$(echo "$json_input" | jq -r '.session_id // empty' 2>/dev/null || echo "")

if [ -z "$session_id" ]; then
    # session_idが取得できない場合はエラーを返す
    echo "Error: Could not extract session_id from SubagentStop hook" >&2
    exit 1
fi

# セッション用のサブエージェントログファイル
subagents_log="${SUBAGENTS_DIR}/${session_id}.jsonl"

# JSONLフォーマットで追記
echo "$json_input" >> "$subagents_log"

# 完了数をカウント
completed_count=$(wc -l < "$subagents_log" 2>/dev/null || echo "0")

# セッションファイルを更新（subagent_completed_countを更新）
sessions_dir="${RUNTIME_DIR}/cc-bar/sessions"
session_file="${sessions_dir}/${session_id}.json"

if [ -f "$session_file" ]; then
    # jq で既存JSONに subagent_completed_count を追加
    jq \
        --argjson count "$completed_count" \
        '.subagent_completed_count = $count' \
        "$session_file" > "${session_file}.tmp" && mv "${session_file}.tmp" "$session_file"
fi

exit 0
