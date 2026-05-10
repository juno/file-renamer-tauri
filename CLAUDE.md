# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

macOS 専用のデスクトップアプリ。フォルダをドロップすると中のファイルを連番（`01.ext`, `02.ext`, ...）にリネームし、元ファイル名の対応表 `filename.txt` を生成する。

## Commands

```bash
npm install            # 依存関係のインストール
npm run tauri dev      # 開発サーバー起動（Vite + Tauri）
npm run release        # リリースビルド・ユニバーサルバイナリ（成果物: src-tauri/target/release/bundle/）
```

テストは現時点で存在しない。Rust のユニットテストを追加する場合は `src-tauri/src/lib.rs` 内に `#[cfg(test)]` モジュールとして書く。

## Architecture

Tauri v2 アプリ。フロントエンドと Rust バックエンドが IPC で通信する。

### フロントエンド (`src/`)

- フレームワークなし（Vanilla JS + Vite）
- `src/main.js` — Tauri の `onDragDropEvent` でドラッグ&ドロップを受け取り、`invoke('rename_files', ...)` でバックエンドを呼び出す。UI 状態（`default` / `dragging` / `loading` / `success` / `error`）を CSS クラスで管理
- `src/style.css` — Apple HIG 準拠のスタイル。状態クラスが border・color を変える

### バックエンド (`src-tauri/src/lib.rs`)

`rename_files(folder_path: String)` が唯一の Tauri コマンド。処理フロー：

1. ファイル一覧取得（隠しファイル・サブディレクトリ・`filename.txt` を除外）
2. アルファベット順ソート
3. **Phase 1**: 全ファイルを `.renamer_tmp_N` に一時リネーム（命名衝突防止）
4. **Phase 2**: 最終連番名（`01.ext` ... または `001.ext` ...）にリネーム
5. `filename.txt` 書き込み（元ファイル名を順番に1行ずつ）

桁数: ファイル数が 100 未満なら 2 桁、100 以上なら 3 桁。

### Tauri 設定

- `src-tauri/tauri.conf.json` — ウィンドウサイズ 480×360、リサイズ不可、バンドル設定
- `src-tauri/capabilities/default.json` — Tauri v2 のパーミッション定義。新しい Tauri API を使う場合はここに追加が必要
