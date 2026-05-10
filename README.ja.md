# File Renamer

フォルダをドロップするだけで、中のファイルを連番にリネームする macOS 用 GUI ユーティリティ。

![Screenshot (Before drop folder)](screenshot-01.png)

![Screenshot (After drop folder)](screenshot-02.png)

## 機能

- フォルダをドラッグ&ドロップするだけで処理完了
- フォルダ内のファイルを `01.ext`, `02.ext`, ... と連番にリネーム（拡張子はそのまま維持）
- リネーム前のファイル名を記録した `filename.txt` をフォルダ内に生成
- 100件以上のファイルには3桁の連番 (`001`, `002`, ...) を使用

### filename.txt の仕様

リネームされた順に元のファイル名を列挙したテキストファイル。
例：`02.mp4` が元は `foo.mp4` だったことを確認できる。

```text
bar.mp4
foo.mp4
```

## 動作環境

- macOS
- [Rust / Cargo](https://www.rust-lang.org/tools/install)
- Node.js 18+

## セットアップ

```bash
npm install
```

## 開発

```bash
npm run tauri dev
```

## ビルド

```bash
npm run tauri build
```

ビルド成果物は `src-tauri/target/release/bundle/` に出力される。

## 技術スタック

- [Tauri v2](https://tauri.app/) — ネイティブアプリフレームワーク
- [Vite](https://vitejs.dev/) — フロントエンドビルドツール
- Vanilla JS / CSS — フロントエンド
- Rust — ファイル操作バックエンド
