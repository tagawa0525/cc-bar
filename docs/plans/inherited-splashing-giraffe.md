# cc-bar: Claude Code Context Window Monitor for Cosmic DE

## Context

Claude Codeのセッション中、コンテキストウィンドウの使用率はターミナル内でしか確認できない。複数セッションを並行して使う場合、各セッションの状況を一覧で把握する手段がない。

Cosmic DEのパネルにアプレットを追加し、全アクティブセッションのコンテキスト使用率をリアルタイムで表示する。

## データソース

Claude Codeの**Status Line機能**がリアルタイムのコンテキストデータを提供する。`~/.claude/settings.json`に設定すると、アシスタントの応答ごとにJSON（stdin経由）が渡される:

```json
{
  "context_window": {
    "used_percentage": 62,
    "remaining_percentage": 38,
    "context_window_size": 200000,
    "total_input_tokens": 124000,
    "total_output_tokens": 4500,
    "current_usage": { "input_tokens": 8500, "output_tokens": 1200, ... }
  },
  "model": { "id": "claude-opus-4-6", "display_name": "Opus" },
  "session_id": "abc123...",
  "cost": { "total_cost_usd": 0.12, "total_duration_ms": 323000 }
}
```

**制約**: サブエージェントの個別コンテキスト使用率はStatus Lineに含まれない。完了数のみ`SubagentStop`フックで取得可能。

## アーキテクチャ

```
Claude Code Session 1 ──> cc-bar-relay.sh ──> $XDG_RUNTIME_DIR/cc-bar/sessions/session1.json
Claude Code Session 2 ──> cc-bar-relay.sh ──> $XDG_RUNTIME_DIR/cc-bar/sessions/session2.json
                                                        │
                                                   inotify watch
                                                        │
                                                  cc-bar (Cosmic Applet)
                                                   Panel: [◉62%] [◉85%] 円グラフ×N
                                                   Popup: 各セッション詳細
```

- **cc-bar-relay.sh**: Status Lineスクリプト。JSONをstdinから読み、`$XDG_RUNTIME_DIR/cc-bar/sessions/{session_id}.json`にatomic write
- **cc-bar**: Cosmic DEアプレット。inotifyでファイル変更を監視し、パネルとポップアップを更新

## UI設計

### パネルバー — セッションごとの円グラフ（ドーナツチャート）

アクティブセッション1つにつき1つの小さなドーナツチャートをパネルに表示する。

```
Panel: [◉] [◉] [◉]   ← 3セッション分の円グラフが横並び
```

- 各ドーナツチャートはコンテキスト使用率を弧の長さで表現（0%=空、100%=満円）
- 色分け: 緑 (<70%), 黄 (70-89%), 赤 (>=90%) ← Cosmicテーマのsuccess/warning/destructiveカラー使用
- チャートのサイズはパネルの高さに合わせる（パネル高さ - パディング）
- セッションなし: グレーアウトした小さい空のドーナツを1つ表示
- ドーナツ中央にモデル名の頭文字を表示（"O"=Opus, "S"=Sonnet, "H"=Haiku）
- 描画は `iced::widget::canvas::Canvas` を使用

### ポップアップ（クリック時）
- セッションごとに: プロジェクトディレクトリ名、モデル名、コンテキスト使用率%、コスト、経過時間、サブエージェント完了数
- 5分間更新がないセッションはstaleとして非表示

## 実装ステップ

### Phase 1: プロジェクト基盤

1. **`flake.nix`作成** — NixOS開発環境（wayland, libxkbcommon, fontconfig, pkg-config等）
2. **`Cargo.toml`更新** — edition `2021`に変更、依存追加:
   - `libcosmic` (git, features: applet, tokio, wayland)
   - `serde`, `serde_json`, `tokio`, `inotify`, `dirs`
3. **最小アプレット** — `cosmic::applet::run()`で起動し、パネルに静的テキスト "CC" を表示するだけの状態
4. **`.desktop`ファイル** — `data/com.github.tagawa.cc-bar.desktop`

### Phase 2: データ層

5. **`src/data.rs`** — Status Line JSONに対応するデシリアライズ型定義
6. **`src/data.rs`** — `SessionStore`（HashMap<session_id, SessionData>）、stale判定、peak使用率計算
7. **テスト** — JSONデシリアライズとSessionStoreのユニットテスト

### Phase 3: ファイル監視

8. **`src/watcher.rs`** — inotifyで`$XDG_RUNTIME_DIR/cc-bar/sessions/`を監視するSubscription
9. **30秒タイマー** — staleセッション除去用の定期Subscription

### Phase 4: Status Lineスクリプト

10. **`scripts/cc-bar-relay.sh`** — `jq`でsession_id抽出、atomic write、Claude Code側にも短い状態表示を返す

### Phase 5: アプレットUI

11. **`src/chart.rs`** — `iced::widget::canvas`によるドーナツチャートウィジェット実装
12. **`src/app.rs` view()** — パネルバーにセッションごとのドーナツチャートを横並び表示
13. **`src/app.rs` view_window()** — ポップアップ（セッション一覧、詳細情報）
14. **`src/app.rs` update()** — メッセージハンドリング、ポップアップ開閉

### Phase 6: サブエージェント追跡

15. **`scripts/cc-bar-subagent-hook.sh`** — SubagentStopフック、jsonl出力
16. **watcher拡張** — subagents/ディレクトリの監視、完了数カウント

### Phase 7: インストール

17. **`~/.claude/settings.json`更新** — statusLineとhooksの設定追加
18. **ビルド・インストール手順** — Makefileまたはjustfile

## 変更対象ファイル

| ファイル | 内容 |
|---|---|
| `Cargo.toml` | 依存追加、edition変更 |
| `flake.nix` | NixOS開発環境 |
| `src/main.rs` | エントリポイント（`cosmic::applet::run`） |
| `src/app.rs` | Application trait実装（view, update, subscription） |
| `src/data.rs` | データ型、SessionStore |
| `src/watcher.rs` | inotifyファイル監視Subscription |
| `src/message.rs` | Messageアイテム |
| `src/chart.rs` | ドーナツチャートのcanvas描画ウィジェット |
| `src/config.rs` | APP_ID等の定数 |
| `scripts/cc-bar-relay.sh` | Status Lineスクリプト |
| `scripts/cc-bar-subagent-hook.sh` | SubagentStopフック |
| `data/com.github.tagawa.cc-bar.desktop` | Cosmicアプレット登録 |

## 検証方法

1. `cargo build`が通ること
2. `cargo test`でデータ層のテストがパスすること
3. `cc-bar-relay.sh`にサンプルJSONをパイプし、`$XDG_RUNTIME_DIR/cc-bar/sessions/`にファイルが生成されること
4. `cc-bar`を起動し、Cosmicパネルに表示されること
5. Claude Codeセッションを開始し、パネルにコンテキスト使用率がリアルタイム更新されること
