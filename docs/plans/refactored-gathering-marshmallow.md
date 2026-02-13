# パネル表示を折れ線グラフに変更

## Context

現在cc-barはパネルにセッションごとの色付き四角形を表示している。モデル色×使用率の濃淡で情報を伝えているが、コンテキスト使用率の推移が見えない。minimon-appletのようなグラフ表示に変更し、使用率の時系列トレンドを視覚化する。

## 方針

minimonと同様にSVG文字列をプログラムで生成し、`cosmic::widget::icon::from_svg_bytes` → `icon()` → `Element`で表示する。Canvas APIは追加feature不要だが、SVGアプローチの方がシンプルで十分。

## 変更ファイル

### 1. `src/data.rs` — 使用率履歴の追加

- `SessionData`に`usage_history: VecDeque<f64>`フィールドを追加
- 定数`MAX_HISTORY_SAMPLES = 21`（minimonと同じサンプル数）
- `update_from_status_line()`で更新のたびに`push_back`、上限超過時に`pop_front`
- `update_session()`も同様に対応
- テスト追加: 履歴蓄積と上限バウンドの検証

### 2. `src/chart.rs` — SVG折れ線グラフ生成

`donut_view()`を削除し、以下に置き換え:

- `colors_for_model(model_type) -> ChartColors` — モデル別の色定義（line/fill/background/frame）
- `generate_line_svg(samples, model_type) -> String` — SVG文字列生成
  - viewBox: `0 0 42 42`、角丸clip（rx=7）
  - Y座標: `41 - (40/100 * value)`（0%=底、100%=上端）
  - X座標: `(index * 2) + 1`（21サンプルが42px幅に収まる）
  - polyline（ストローク）+ polygon（塗りつぶし、底辺まで閉じる）
- `line_chart_view(samples, model_type, size) -> Element` — SVGをwidgetに変換
  - `cosmic::widget::icon::from_svg_bytes` → `icon()` → `.width()/.height()` → `Element`

色定義:
| モデル | Line | Fill | Frame |
|--------|------|------|-------|
| Opus | `#FF8C1A` | `#FF8C1A60` | `#FF8C1A40` |
| Sonnet | `#3366E6` | `#3366E660` | `#3366E640` |
| Haiku | `#1AB34D` | `#1AB34D60` | `#1AB34D40` |

背景: `#1A1A1AE0`（全モデル共通）

### 3. `src/app.rs` — view()の呼び出し変更

`view()`メソッド内の`donut_view()`呼び出しを`line_chart_view()`に変更:

```rust
// Before
crate::chart::donut_view::<Message>(session.context_used_percent, model_type_str, chart_size)
// After
crate::chart::line_chart_view::<Message>(&session.usage_history, model_type_str, chart_size)
```

レイアウトロジック（Row/Column、spacing、button wrap）は変更なし。

## 変更しないファイル

- `Cargo.toml` — 新規依存なし（VecDequeは標準ライブラリ）
- `src/watcher.rs` — メッセージ送信ロジックに変更なし
- `src/message.rs` — メッセージ型に変更なし

## コミット順序（TDD）

1. **RED**: `data.rs`に履歴テスト追加（コンパイルエラー/テスト失敗）
2. **GREEN**: `SessionData`に`usage_history`追加、更新ロジック実装
3. **RED→GREEN**: `chart.rs`をSVG折れ線グラフに書き換え、`app.rs`の呼び出し変更

## 検証方法

1. `cargo build` — コンパイル確認
2. `cargo test` — data.rsのユニットテスト通過
3. 実機テスト: cc-barを起動し、Claude Codeセッションを開始してパネルに折れ線グラフが表示されることを確認
