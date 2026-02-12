#!/usr/bin/env just --justfile

set shell := ["bash", "-c"]

# 開発環境の初期化
init:
    #!/usr/bin/env bash
    echo "cc-bar: Claude Code Context Window Monitor"
    echo ""
    echo "依存インストール中..."
    [ -f flake.nix ] && nix flake update || true
    echo "✓ 初期化完了"

# Nix開発シェルで実行
nix-shell cmd="":
    @if [ -z "{{ cmd }}" ]; then \
        nix develop; \
    else \
        nix develop --command {{ cmd }}; \
    fi

# ビルド
build: init
    @echo "Building cc-bar..."
    nix develop --command cargo build --release

# テスト実行
test:
    @echo "Running tests..."
    nix develop --command cargo test --lib

# フォーマット確認
fmt-check:
    @cargo fmt -- --check

# フォーマット実行
fmt:
    @cargo fmt

# インストール（ユーザーローカル）
install: build
    #!/usr/bin/env bash
    set -euo pipefail

    INSTALL_DIR="${HOME}/.local/bin"
    mkdir -p "$INSTALL_DIR"

    # NixOS: dlopen用ライブラリパスをバイナリに焼き込む
    nix develop --command bash -c \
        'patchelf --set-rpath "$(echo "$LD_LIBRARY_PATH" | sed "s/:$//")" target/release/cc-bar'

    cp target/release/cc-bar "$INSTALL_DIR/"
    cp scripts/cc-bar-relay.sh "$INSTALL_DIR/"
    cp scripts/cc-bar-subagent-hook.sh "$INSTALL_DIR/"

    echo "✓ Installed to $INSTALL_DIR"
    echo ""
    echo "Next: Run 'just configure' to set up Claude Code settings"

# Claude Codeの設定
configure:
    #!/usr/bin/env bash
    set -euo pipefail

    CLAUDE_DIR="${HOME}/.claude"
    SETTINGS="${CLAUDE_DIR}/settings.json"
    SCRIPTS_DIR="$(pwd)/scripts"

    if [ ! -f "$SETTINGS" ]; then
        echo "Error: $SETTINGS not found"
        echo "Please create it first with Claude Code"
        exit 1
    fi

    echo "Claude Code設定を更新中..."

    # backup
    cp "$SETTINGS" "$SETTINGS.bak"

    # statusLineスクリプトを設定
    STATUS_LINE_SCRIPT="$SCRIPTS_DIR/cc-bar-relay.sh"
    SUBAGENT_HOOK_SCRIPT="$SCRIPTS_DIR/cc-bar-subagent-hook.sh"

    # jqで設定を追加（既存の設定を保持）
    jq \
        --arg status_line "$STATUS_LINE_SCRIPT" \
        --arg subagent_hook "$SUBAGENT_HOOK_SCRIPT" \
        '.statusLine = {"type": "command", "command": $status_line} |
         .hooks //= {} |
         .hooks.SubagentStop = [{"hooks": [{"type": "command", "command": $subagent_hook}]}]' \
        "$SETTINGS" > "$SETTINGS.tmp" && \
        mv "$SETTINGS.tmp" "$SETTINGS"

    echo "✓ 設定を更新しました"
    echo ""
    echo "設定内容:"
    jq '.statusLine, .hooks' "$SETTINGS"
    echo ""
    echo "Claude Codeを再起動して、設定を有効化してください"

# デスクトップファイルのインストール
install-desktop:
    #!/usr/bin/env bash
    set -euo pipefail

    DESKTOP_DIR="${HOME}/.local/share/applications"
    mkdir -p "$DESKTOP_DIR"

    cp data/com.github.tagawa.cc-bar.desktop "$DESKTOP_DIR/"

    echo "✓ Desktop file installed to $DESKTOP_DIR"
    echo "cc-barをCosmicパネルに追加してください"

# すべてのセットアップ
setup: install install-desktop configure
    #!/usr/bin/env bash
    echo ""
    echo "========================================="
    echo "✓ cc-bar セットアップ完了"
    echo "========================================="
    echo ""
    echo "使用方法:"
    echo "1. Claude Codeを再起動"
    echo "2. Cosmic Desktopパネルを右クリック"
    echo "3. 'cc-bar'アプレットを追加"
    echo ""
    echo "トラブルシューティング:"
    echo "- ログの確認: $XDG_RUNTIME_DIR/cc-bar/sessions/"
    echo "- デバッグ: cc-bar --help (未実装)"

# クリーンアップ
clean:
    @cargo clean
    @echo "✓ Cleaned"

# すべてのフェーズの確認
check-all: fmt-check test
    @echo "✓ All checks passed"

# ローカルランナー（Cosmic DEがない環境用）
run-mock:
    @echo "Running in mock mode (no Cosmic DE)"
    @# MOCKモードでのテストはまだ未実装

# デフォルトターゲット
default:
    @just --list
