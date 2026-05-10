# File Renamer

A minimal macOS GUI utility that renames all files in a dropped folder to sequential numbers.

![Screenshot (Before drop folder)](screenshot-01.png)

![Screenshot (After drop folder)](screenshot-02.png)

## Features

- Drop a folder onto the window — that's it
- Files are renamed to `01.ext`, `02.ext`, ... preserving their original extensions
- A `filename.txt` is created inside the folder, listing the original filenames in order
- Automatically switches to 3-digit numbering (`001`, `002`, ...) for 100+ files

### filename.txt

A plain-text file that maps the new sequential names back to the originals.
Example: given `bar.mp4` and `foo.mp4`, after renaming:

```text
bar.mp4
foo.mp4
```

This tells you `01.mp4` was `bar.mp4` and `02.mp4` was `foo.mp4`.

## Requirements

- macOS
- [Rust / Cargo](https://www.rust-lang.org/tools/install)
- Node.js 18+

## Setup

```bash
npm install
```

## Development

```bash
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

The distributable is output to `src-tauri/target/release/bundle/`.

- **DMG installer**: `src-tauri/target/release/bundle/dmg/`
- **App bundle**: `src-tauri/target/release/bundle/macos/File Renamer.app`

To install, open the DMG and drag `File Renamer.app` to your Applications folder, or copy the `.app` bundle directly.

## Tech Stack

- [Tauri v2](https://tauri.app/) — native app framework
- [Vite](https://vitejs.dev/) — frontend build tool
- Vanilla JS / CSS — frontend
- Rust — file operation backend

## License

[MIT](LICENSE.md)
