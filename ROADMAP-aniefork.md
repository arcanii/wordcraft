# WordCraft aniefork roadmap

This is the fork's own record: [arcanii/wordcraft](https://github.com/arcanii/wordcraft), a fork of
[storytold/wordcraft](https://github.com/storytold/wordcraft). The sister fork
[arcanii/pdfcraft](https://github.com/arcanii/pdfcraft) keeps the same kind of file, and the two
follow the same rules ([`AGENTS-aniefork.md`](AGENTS-aniefork.md)).

Upstream's [ROADMAP.md](ROADMAP.md), `docs/`, `CLAUDE.md` and `AGENTS.md` are kept **identical to
upstream** here, so merging upstream doesn't conflict on bookkeeping. Work done in the fork is
logged in this file instead.

- **Fork work:** add a line to the log below, and keep the tables true. Do not edit `ROADMAP.md`
  or upstream's `docs/`.
- **A branch meant as an upstream pull request:** follow upstream's rules there (`CLAUDE.md`),
  including its progress documents. Cut the branch from `upstream/main`, not from the fork's `main`.

## Sync status

| | |
|---|---|
| Fork `main` is based on | upstream `e4b7d15` (WordCraft 0.5.0), 2026-10-11 |
| Last merge of upstream | none yet |
| Fork carries | 2 changes upstream lacks (below) |

## What the fork carries

Changes in the fork that upstream does not have. Update the last column when one is offered or
merged; delete the row once a merge of upstream brings it back.

| Change | Where | Upstream |
|---|---|---|
| **Caret glide:** the caret slides to its new place instead of jumping, like the caret in Word for Windows. First version; the timing is estimated, not yet measured against Word. | `crates/ui-egui/src/aniefork/caret.rs`, two hooks in `canvas.rs` (`520f021`) | Not offered |
| **The paste-picture test passes on Windows.** `paste_inserts_clipboard_pixels` gave `names_picture_files` the path `/tmp/b.JPG`, which Windows doesn't call absolute, so the test failed there. It now uses the system's own absolute path, and on Windows also checks drive, server and `file:///C:/…` forms. Test only: the app behaves as before. | the tests in `crates/ui-egui/src/paste_picture.rs` | Not offered; upstream has the same failure, so it is a candidate for a pull request |

## Caret glide

Word for Windows doesn't move its caret instantly: type, press an arrow key or click, and the bar
slides to the new place. Upstream WordCraft jumps. The fork glides.

- **What glides:** a caret that moved because the selection or the document changed: typing,
  Backspace, arrows, Home/End, Enter (diagonally to the next line), a click, a formatting change that
  reflows the line, the caret inside an equation. A new move takes over from wherever the caret is
  drawn, so quick typing reads as one continuous motion. The bar stays lit while it moves and
  blinks again once it rests.
- **What doesn't:** a caret that moved because the view changed (zoom, a resized window, scrolling,
  another document), a caret that wasn't drawn the frame before (a selection, the canvas without
  focus), and jumps that start or end off screen (Ctrl+End, Page Down). Those appear in place.
- **Timing:** 0.12 s with a decelerating (ease-out cubic) curve, the same at any frame rate.
  `WORDCRAFT_CARET_GLIDE_MS` overrides the duration: `0` turns the glide off, a large value (up
  to 10000) slows it down enough to study.
- **Off switches:** the glide stands down when egui's animations are off
  (`style.animation_time == 0`). The headless test harness (`egui_kittest`, so `ui_shot` too) turns
  them off, so scripted screenshots never catch a caret in flight.
- **Clean room:** the behaviour is from watching Word's caret as a user; nothing was read from the
  Word program. The duration and curve are our estimate.
- **Tests:** `cargo test -p wordcraft-ui-egui aniefork` (the motion, frame-rate independence,
  retargeting, view changes, hostile numbers, and the caret gliding in the real canvas).

### Open
- Measure Word's caret (duration, curve, how long jumps behave) from a capture of its window and
  set `GLIDE_SECS` and the curve to match.
- A setting to turn it off (Word: Options › Advanced › "Provide feedback with animation"), and
  following the system's reduced-motion setting. Today: the environment variable above.

## Hooks in upstream files

Everything else the fork adds is in files of its own. After merging upstream, check these are
still there:

| File | Hook |
|---|---|
| `crates/ui-egui/src/lib.rs` | `pub mod aniefork;` |
| `crates/ui-egui/src/canvas.rs`, `show`, the equation caret | `crate::aniefork::caret::glide(…)` gives the bar to draw; it is drawn when the blink is on **or** it is gliding |
| `crates/ui-egui/src/canvas.rs`, `show`, the text caret | the bar is computed first, passed through `crate::aniefork::caret::glide(…)`, and drawn when the blink is on **or** it is gliding |

## Syncing with upstream

```sh
git fetch upstream
git merge upstream/main
```

Sync often: upstream lands dozens of commits a day, and small merges are easy. What can still
conflict:

- **The hooks above**, when upstream edits the caret drawing in `canvas.rs`. Take upstream's code
  and put the hook back.
- **Interface strings:** upstream adds languages with a test that every `tl!("…")` string is
  translated. A string the fork adds must be translated in every catalog (the fork has none so far).
- **`ATTRIBUTION.md`:** when both sides add assets (the fork has none so far).

After a merge, run `cargo xtask ci` before committing it, and update the tables above.

## Building

The workspace says `rust-version = "1.90"`, but egui 0.36 needs Rust 1.95 or newer; an older
toolchain fails with a dependency error (pdfcraft's fork found the same and carries a fix).
`cargo xtask ci` also needs the `wasm32-unknown-unknown` target.

Upstream runs its test suite on FreeBSD only (`.github/workflows/freebsd.yml`), so a test that
fails on Windows goes unnoticed there. The fork is developed on Windows: run the whole gate, and
don't set `WGPU_BACKEND` for it (a graphics test reads eframe's default backends, which follow it).

## Log

Newest first.

- **2026-10-11 (M2, paste-picture test on Windows):** `cargo xtask ci` failed on Windows at one
  upstream test: `paste_inserts_clipboard_pixels` expected the text `/tmp/b.JPG` to name a copied
  file, but a path needs a drive or a server to be absolute on Windows. The test now uses each
  system's own absolute path and covers the Windows forms. It was the only test in the workspace
  that failed on Windows; the gate is green (1,078 tests).
- **2026-10-11 (M14, caret glide):** The caret glides to its new place instead of jumping, like
  Word's on Windows: 0.12 s, decelerating, from wherever it is drawn (so typing reads as one
  motion), staying lit on the way. View changes, scrolling and off-screen jumps don't glide. All of
  it is in the new `aniefork` module of `ui-egui`, with two hooks in `canvas.rs`. Checked in a
  frame-by-frame headless render and by seven tests. This file and `AGENTS-aniefork.md` were started.
