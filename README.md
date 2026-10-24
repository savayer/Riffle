# Riffle

A fast, minimal photo and video viewer for macOS. Double-click a file in Finder and flip through
everything else in that folder with the arrow keys or a trackpad swipe — like the Windows Photos app.

Built with Tauri 2 (Rust) and React + TypeScript. The app is ~4 MB; the interface follows the system language (English or Russian).

## Download

Get the latest `.dmg` from [Releases](https://github.com/savayer/Riffle/releases/latest) (universal: Apple Silicon and Intel),
open it and drag Riffle to Applications. Releases are signed with a Developer ID and notarized by Apple.

## Features

- Opens the file you double-clicked and lets you browse every other photo and video in the same folder
- Folder order matches Finder: natural sort (`IMG_2` before `IMG_10`), case-insensitive, hidden files skipped
- Adjacent images (±2) are preloaded, so switching is instant and flicker-free
- Videos autoplay with native controls
- Broken or unsupported files show a placeholder; browsing keeps working
- Window title shows `name — 12 / 148`
- "Show in Finder" in the right-click menu (the standard WebKit items stay) and on <kbd>⌘</kbd><kbd>R</kbd>
- Launched without a file: an "Open…" button and drag-and-drop

**Formats:** jpg, jpeg, png, gif, webp, heic, heif, bmp, tiff, mp4, mov, m4v, webm.
HEIC/HEIF are rendered natively by WebKit.

## Controls

| Action | Input |
| --- | --- |
| Previous / next | <kbd>←</kbd> / <kbd>→</kbd>, two-finger horizontal swipe, on-screen ‹ › buttons |
| First / last | <kbd>Home</kbd> / <kbd>End</kbd> |
| Play / pause video | <kbd>Space</kbd> |
| Show in Finder | <kbd>⌘</kbd><kbd>R</kbd> or right-click |

Browsing stops at the first and last file; it doesn't wrap around. The ‹ › buttons appear when you move the mouse.

## Build and install

Requirements: macOS, [Xcode Command Line Tools](https://developer.apple.com/xcode/resources/),
[Rust](https://rustup.rs) (stable), Node.js 24+.

```bash
npm install
npm run tauri build
ditto src-tauri/target/release/bundle/macos/Riffle.app /Applications/Riffle.app
```

To build a `.dmg` instead:

```bash
CI=true npm run tauri build -- --bundles dmg
# → src-tauri/target/release/bundle/dmg/Riffle_<version>_aarch64.dmg
```

`CI=true` skips the Finder AppleScript that only lays out the DMG window, which fails without
Automation permission.

A local build is ad-hoc signed: fine on your own Mac, but a copy sent elsewhere is blocked by Gatekeeper.

### Release (signed + notarized)

Requires a Developer ID Application certificate in the keychain and an App Store Connect API key
(Users and Access → Integrations → Team Keys, access: Developer).

```bash
cp .env.example .env.local   # fill in; .env.local is gitignored
npm run build:release        # universal DMG, signed, app + DMG notarized and stapled
gh release create v<version> src-tauri/target/universal-apple-darwin/release/bundle/dmg/*.dmg
```

The first notarization with a new certificate can take up to an hour; later ones take minutes.

### Make Riffle the default viewer

In Finder, select a file → <kbd>⌘</kbd><kbd>I</kbd> (Get Info) → "Open with" → Riffle → "Change All…".
macOS stores this per file type, so repeat it once for each type you use (jpg, png, heic, mp4, …).

## Development

```bash
npm run tauri dev                              # empty window
npm run tauri dev -- -- /absolute/path/to.jpg  # open a file (the path must be absolute)

npm test                                       # frontend logic (node --test)
cd src-tauri && cargo test                     # folder listing and natural sort
```

Opening files from Finder (`RunEvent::Opened`) only happens in a bundled app. To test it, build and run
`open -a /path/to/Riffle.app some-file.jpg`.

## How it works

- **Opening a file:** macOS delivers it as `RunEvent::Opened`, often before the frontend has loaded.
  Rust keeps the path in managed state and the frontend picks it up with `get_initial_file`. When the app
  is already running, an `open-file` event switches the current window.
- **File access:** the asset protocol starts with an empty scope. `list_media` grants only the folder of
  the opened file, plus the file itself, at runtime.
- **Context menu:** WebKit's menu can't be extended from JavaScript. Rust adds `willOpenMenu:withEvent:` to
  wry's `WKWebView` subclass and appends the "Show in Finder" item there.

| Path | Role |
| --- | --- |
| `src-tauri/src/lib.rs` | Commands, open-file handling, asset scope |
| `src-tauri/src/media.rs` | Folder listing, media filter, natural sort |
| `src-tauri/src/context_menu.rs` | "Show in Finder" in the native context menu |
| `src/App.tsx` | State, keyboard, swipe, window title |
| `src/Viewer.tsx` | Image/video rendering, preloading, placeholder |
| `src/swipe.ts`, `src/nav.ts` | Pure navigation logic (tested) |

## Not yet

Zoom, delete / move to Trash, editing, watching the folder for changes, multiple windows, Intel builds
(build with `--target universal-apple-darwin` after `rustup target add x86_64-apple-darwin`).
