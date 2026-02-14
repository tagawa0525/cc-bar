# cc-bar: Claude Code Context Window Monitor for Cosmic DE

複数のClaude Codeセッションのコンテキストウィンドウ使用率をリアルタイムでCosmicデスクトップパネルに表示します。

## 機能

- **パネル表示**: アクティブセッションごとのコンテキスト使用率を折れ線グラフで表示
  - モデル種別で色分け: Opus (紫)、Sonnet (青)、Haiku (緑)、Unknown (灰)
- **ポップアップ**: クリックでセッション詳細を表示
  - プロジェクト名、モデル、コンテキスト使用率
  - コスト、経過時間、ピーク使用率
  - サブエージェント完了数
- **自動監視**: Claude CodeのStatus Lineで自動更新
- **Stale除去**: 2分間更新がないセッションは自動削除

## 要件

- Rust 1.70+
- jq (Status Line / Hook スクリプトで使用)
- Nix (オプション、開発環境用)
- Claude Code (Version 1.40+)
- Cosmic Desktop Environment

## インストール

### クイックセットアップ

```bash
just setup
```

このコマンドが以下を自動実行します:
1. ビルド
2. `~/.local/bin`へのインストール
3. デスクトップファイル登録
4. Claude Code設定更新

### 手動セットアップ

```bash
# ビルド
just build

# インストール
just install

# デスクトップファイル
just install-desktop

# Claude Code設定
just configure
```

## 設定

### ~/.claude/settings.json の設定例

```json
{
  "statusLine": { "type": "command", "command": "~/.local/bin/cc-bar-relay.sh" },
  "hooks": {
    "SubagentStop": [
      {
        "matcher": "",
        "hooks": [
          { "type": "command", "command": "~/.local/bin/cc-bar-subagent-hook.sh" }
        ]
      }
    ],
    "SessionEnd": [
      {
        "matcher": "",
        "hooks": [
          { "type": "command", "command": "~/.local/bin/cc-bar-session-cleanup.sh" }
        ]
      }
    ]
  }
}
```

### 開発環境

```bash
# Nixシェルで開発
nix develop

# または
just nix-shell

# ビルド
just build

# テスト
just test

# フォーマット
just fmt
```

## アーキテクチャ

```
Claude Code Session ──> cc-bar-relay.sh ──> $XDG_RUNTIME_DIR/cc-bar/sessions/{session_id}.json
                                                      ↓
                                                  inotify watch
                                                      ↓
                                                cc-bar (Cosmic Applet)
                                          Panel: [📈62%] [📈85%]  ←  折れ線グラフ×N
                                          Popup: セッション詳細
```

### コンポーネント

| ファイル | 役割 |
|---------|------|
| `src/main.rs` | エントリポイント |
| `src/app.rs` | Application trait実装 |
| `src/data.rs` | Status Line JSON型、SessionStore |
| `src/watcher.rs` | inotifyファイル監視Subscription |
| `src/chart.rs` | SVG折れ線グラフ生成 |
| `scripts/cc-bar-relay.sh` | Status Lineスクリプト |
| `scripts/cc-bar-subagent-hook.sh` | SubagentStopフック |

## 使用方法

1. **Claude Codeを再起動** (設定を反映)
2. **Cosmicパネルを右クリック**
3. **"CC Bar"を追加** (アプレットリスト内)
4. **複数セッション実行**
5. **パネルに折れ線グラフ表示** (リアルタイム更新)
6. **クリックでセッション詳細表示**

## トラブルシューティング

### セッションが表示されない

```bash
# セッションディレクトリ確認
ls $XDG_RUNTIME_DIR/cc-bar/sessions/

# ファイルが存在するか
cat $XDG_RUNTIME_DIR/cc-bar/sessions/*.json | jq .
```

### スクリプトエラー

```bash
# Status Lineスクリプトのテスト
echo '{"context_window":{"used_percentage":50,...},"session_id":"test",...}' | \
  ~/.local/bin/cc-bar-relay.sh

# エラーログ確認
journalctl -xe
```

### Nix環境の問題

```bash
# キャッシュクリア
rm flake.lock
nix flake update

# デバッグビルド
nix develop --command cargo build -vv
```

## 仕様

### Status Line JSON形式

```json
{
  "context_window": {
    "used_percentage": 62,
    "remaining_percentage": 38,
    "context_window_size": 200000,
    "total_input_tokens": 124000,
    "total_output_tokens": 4500
  },
  "model": {
    "id": "claude-opus-4-6",
    "display_name": "Opus"
  },
  "session_id": "abc123...",
  "cost": {
    "total_cost_usd": 0.12,
    "total_duration_ms": 323000
  }
}
```

### SubagentStop フック形式

```json
{
  "session_id": "abc123...",
  "subagent_id": "xyz789...",
  "timestamp": 1708928123
}
```

## 開発

### ファイル構成

```
cc-bar/
├── src/
│   ├── main.rs           # アプレット起動
│   ├── app.rs            # Application実装
│   ├── data.rs           # データ型とSessionStore
│   ├── watcher.rs        # ファイル監視
│   ├── chart.rs          # UIウィジェット
│   ├── message.rs        # メッセージ型
│   ├── config.rs         # 定数
│   └── lib.rs            # ライブラリ公開
├── scripts/
│   ├── cc-bar-relay.sh             # Status Lineスクリプト
│   └── cc-bar-subagent-hook.sh     # SubagentStopフック
├── data/
│   └── com.github.tagawa.cc-bar.desktop
├── flake.nix             # Nix開発環境
├── Cargo.toml
├── justfile              # ビルド/インストール自動化
└── README.md
```

### テスト

```bash
# ユニットテスト
just test

# 統合テスト（Status Lineシミュレーション）
cat > /tmp/test_status.json << 'EOF'
{...}
EOF
cat /tmp/test_status.json | scripts/cc-bar-relay.sh
```

## ライセンス

MIT

## 謝辞

- [Pop!_OS](https://pop.system76.com/) Cosmic Desktop Environment
- [libcosmic](https://github.com/pop-os/libcosmic) - Rust UIツールキット
- Claude Code Status Line API

## サポート

問題報告やフィードバックは GitHub Issues へ
