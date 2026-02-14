#!/usr/bin/env bash
# cc-bar-session-cleanup.sh - Claude Code SessionEnd Hook
# セッション終了時にセッションファイルとサブエージェントログを削除
# 用法: ~/.claude/settings.json の hooks.SessionEnd に設定

set -euo pipefail

RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"

# stdinからJSONを読み込む
json_input=$(cat)

# session_idを抽出
session_id=$(echo "$json_input" | jq -r '.session_id // empty' 2>/dev/null || echo "")

if [ -z "$session_id" ]; then
    exit 0
fi

# セッションファイルを削除
rm -f "${RUNTIME_DIR}/cc-bar/sessions/${session_id}.json"
rm -f "${RUNTIME_DIR}/cc-bar/subagents/${session_id}.jsonl"

exit 0
