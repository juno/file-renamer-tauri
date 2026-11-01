# CLAUDE.md

## Overview

macOS 専用のデスクトップアプリ。フォルダをドロップすると中のファイルを連番（`01.ext`, `02.ext`, ...）にリネームし、元ファイル名の対応表 `filename.txt` を生成する。「元に戻すモード」では `filename.txt` を使って元の名前に戻す。

## Commands

```bash
pnpm install            # 依存関係のインストール
pnpm run tauri dev      # 開発サーバー起動（Vite + Tauri）
pnpm run release        # リリースビルド・ユニバーサルバイナリ（成果物: src-tauri/target/release/bundle/）
cd src-tauri && cargo test   # Rust のテスト（CI は fmt・clippy も実行）
```

**README の同期**: `README.md` を編集した場合は必ず `README.ja.md` も同じ内容になるよう日本語訳を更新すること。両ファイルは常に同期を保つ。

**コードレビュー**: `CODING_STANDARDS.md` の基準で行う。

## Architecture

Tauri v2 アプリ。フロントエンドと Rust バックエンドが IPC で通信する。

- `src/main.js` — `onDragDropEvent` でドロップを受け取り、チェックボックスの状態で `rename_files` / `restore_files` を `invoke` する。UI 状態（`default` / `dragging` / `loading` / `success` / `error`）は CSS クラスで管理。フレームワークなし（Vanilla JS + Vite）
- `src-tauri/src/lib.rs` — Tauri コマンド `rename_files` / `restore_files` と、その単体テスト（`#[cfg(test)]` モジュール）。再ドロップ時は `filename.txt` を読んで既存の対応を引き継ぐ（`build_entries`）
- `src-tauri/capabilities/default.json` — Tauri v2 のパーミッション定義。新しい Tauri API を使う場合はここに追加が必要
