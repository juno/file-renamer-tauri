# File Renamer

フォルダをドロップするだけで、中のファイルを連番にリネームする macOS 用 GUI ユーティリティ。

![Screenshot (Before drop folder)](screenshot-01.png)

![Screenshot (After drop folder)](screenshot-02.png)

## 機能

- フォルダをドラッグ&ドロップするだけで処理完了
- フォルダ内のファイルを `01.ext`, `02.ext`, ... と連番にリネーム（拡張子はそのまま維持）
- リネーム前のファイル名を記録した `filename.txt` をフォルダ内に生成
- 100件以上のファイルには3桁の連番 (`001`, `002`, ...) を使用
- ファイルを追加した後に同じフォルダを再ドロップすると、既存ファイルは元のファイル名を引き継ぎ、新規ファイルが末尾にソート順で追加される

### filename.txt の仕様

リネームされた順に元のファイル名を列挙したテキストファイル。
例：`bar.mp4` と `foo.mp4` をリネームすると：

```text
bar.mp4
foo.mp4
```

これにより `01.mp4` が元の `bar.mp4`、`02.mp4` が元の `foo.mp4` だったとわかる。

同じフォルダを再ドロップした場合、`filename.txt` を読み込んで元のファイル名の対応を復元し、新規ファイルは既存ファイルの後ろに追加される。

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
npm run release
```

ユニバーサルバイナリ（Apple Silicon + Intel 対応）を生成し、`src-tauri/target/universal-apple-darwin/release/bundle/` に出力される。

- **DMG インストーラ**: `src-tauri/target/universal-apple-darwin/release/bundle/dmg/`
- **App バンドル**: `src-tauri/target/universal-apple-darwin/release/bundle/macos/File Renamer.app`

インストールは DMG を開いて `File Renamer.app` をアプリケーションフォルダにドラッグするか、`.app` バンドルを直接コピーする。

## 技術スタック

- [Tauri v2](https://tauri.app/) — ネイティブアプリフレームワーク
- [Vite](https://vitejs.dev/) — フロントエンドビルドツール
- Vanilla JS / CSS — フロントエンド
- Rust — ファイル操作バックエンド

## ライセンス

[MIT](LICENSE.md)
