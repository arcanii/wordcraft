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
| **The caret glides and fades like Word's on Windows** instead of jumping and blinking hard: 100 ms along Word's curve for keys and typing, a fading blink. Calibrated against Word (see below). File › Options › Animation turns it off. | `crates/ui-egui/src/aniefork/` (`caret.rs`, `options.rs`, `strings.rs`); hooks listed below (`520f021`, `5cccc23`, then the Options commit) | Not offered |
| **The paste-picture test passes on Windows.** `paste_inserts_clipboard_pixels` gave `names_picture_files` the path `/tmp/b.JPG`, which Windows doesn't call absolute, so the test failed there. It now uses the system's own absolute path, and on Windows also checks drive, server and `file:///C:/…` forms. Test only: the app behaves as before. | the tests in `crates/ui-egui/src/paste_picture.rs` | Not offered; upstream has the same failure, so it is a candidate for a pull request |

## The caret: Word's glide and fade

Upstream's caret jumps to its new place and blinks hard on and off. The fork's moves and blinks
the way Word's does on Windows.

### What Word does (measured 2026-10-11)

Word for Windows 16.0.20326 (Office 2021), 100% zoom, a 144 Hz display at 150% scaling.

| | Measured |
|---|---|
| Glide: how long | **100 ms** (14 consecutive frames 6.94 ms apart, the last at 97 ms) |
| Glide: the curve | **`cubic-bezier(0, 0, 0, 1)`**. Fraction of the way at each frame: 0.369 0.529 0.640 0.724 0.790 0.843 0.885 0.918 0.945 0.965 0.980 0.991 0.997 1.0; the curve gives 0.368 0.527 0.638 0.722 0.788 0.840 0.882 0.916 0.943 0.964 0.979 0.990 0.997 1.0 |
| Glide: distance | the same time and curve for 6 px (a typed space), 11–13 px (one character), 50 px (a line up or down) and 372 px (Home, End) |
| What glides | arrow keys, Home and End, typing a letter or a space, Backspace |
| What doesn't | a mouse click (the caret is drawn where it was clicked); a move made through automation (it jumps and is solid at once) |
| A second key before it arrives | the caret snaps to the end of the first move and glides from there (seen 45, 60 and 90 ms after the first key, and with a key held at ~31 ms a press) |
| The blink | on 530 ms, off 530 ms (the system's caret blink time); each change is a **linear fade of 100 ms** |
| Opacity on a move | a key or a click turns the caret on: linearly from however visible it was to solid over the same 100 ms. From a caret that was off, the first third of a long glide is practically unseen |
| With Word's animations off | it jumps and blinks hard. Word turned them off by itself part-way through two sessions (cause unknown; possibly the load of being captured) |

**How it was measured (clean room).** A separate Word instance was started through its automation
interface with a five-line synthetic document that was never saved. Its window, and only its
window, was captured by handle about 50 to 70 times a second, and the caret was found by
comparing each frame with a caret-free picture of the page. Keys and clicks were delivered as
messages to that instance's own document window, not as system input. About 220 moves were
captured; the caret animated in about 110 of them (the rest were automation moves, or came after
Word had turned its animations off).
Nothing was read from the Word program. The scripts and the numbers are in `plan/word/caret-lab/`
and the few screenshots in `plan/word/screenshots/caret/` (both local only, never committed).

**Not measured:** other zoom levels and refresh rates; Ctrl+arrow, Page Up/Down and other keys that
need a modifier; moves that start or end off screen; input methods; a click while the caret is lit
(taken to leave it lit, as a key does).

### What WordCraft does

- **Keys and typing:** the caret glides 100 ms along Word's curve, from where it was (or, mid-glide,
  from where it was heading), and turns on from however visible it was.
- **Clicks:** no glide; the caret turns on at its new place.
- **Commands from a script, the control channel or MCP:** no glide, no fade, as Word's automation.
- **The blink:** 530 ms on, 530 ms off, each change a 100 ms fade.
- **Our own choices, where Word wasn't measured:** nothing glides when the view changed (zoom, a
  resized window, scrolling, another document), when the caret wasn't drawn the frame before (a
  selection, the canvas without focus), or when a move starts or ends off screen.
- **The setting:** File › Options › Animation › "Animate the caret (it glides and fades)", on by
  default. Off, the caret jumps and blinks hard, exactly as upstream. It is the command
  `ui.caretAnimation` (`{"value": true|false}` sets it, no value reads it; the reply is
  `{"caretAnimation": …}`), and it is saved with the interface settings (`ui.json`, under
  `aniefork`), so builds without the fork ignore it.
- **Every platform the same.** The switch is in the application, not tied to an operating
  system's own animation setting, because WordCraft runs on macOS, Windows, Linux, BSD and the
  web and the option has to mean the same thing on each (the owner's decision, 2026-10-11). The
  caret code is plain Rust and egui with nothing platform-specific in it, and the wasm build is
  part of the gate. The desktop apps save the setting; the web build doesn't save any interface
  setting between visits (upstream's gap, the same for the theme and the language).
- **Other off switches:** egui's animations off (`style.animation_time == 0`), which the headless
  test harness (`egui_kittest`, so `ui_shot` too) does, so scripted screenshots never catch a
  caret in flight; and `WORDCRAFT_CARET_GLIDE_MS=0`. Any other value of that variable sets the
  glide and fade time (up to 10000), for looking at the motion slowed down.
- **The section's text** is translated into every interface language in the fork's own table
  (`aniefork/strings.rs`), not in upstream's catalogs. The translations are ours and haven't been
  read by native speakers.
- **Tests:** `cargo test -p wordcraft-ui-egui aniefork` (Word's measured frames against the curve,
  frame-rate independence, the second-key rule, clicks and scripts, the blink's fades, view
  changes, hostile numbers, a key gliding the caret in the real canvas, the setting as a command
  and on the Options page, and every text translated and covered by the interface fonts).

### Open
- The cases listed under "Not measured".
- A native speaker's read of the nine translations.

## Hooks in upstream files

Everything else the fork adds is in files of its own. After merging upstream, check these are
still there:

| File | Hook |
|---|---|
| `crates/ui-egui/src/lib.rs` | `pub mod aniefork;` |
| `crates/ui-egui/src/lib.rs`, `UiState` | the field `pub aniefork: aniefork::Prefs` (last in the struct) and `aniefork: aniefork::Prefs::default()` in its `Default` |
| `crates/ui-egui/src/lib.rs`, `ui_command` | the match's last arm is `_ => return aniefork::ui_command(self, id, p)` (upstream: `_ => return None`) |
| `crates/ui-egui/src/backstage.rs`, `options_page` | `crate::aniefork::options::section(app, ui);` just before the Agents heading |
| `crates/ui-egui/src/canvas.rs`, `show`, the equation caret | `crate::aniefork::caret::look(…)` gives the bar and its opacity; it is drawn at that opacity (`gamma_multiply`) when the opacity is above 0 |
| `crates/ui-egui/src/canvas.rs`, `show`, the text caret | the bar is computed first, passed through `crate::aniefork::caret::look(…)` with `since` and `on`, and drawn at the returned opacity |

## Syncing with upstream

```sh
git fetch upstream
git merge upstream/main
```

Sync often: upstream lands dozens of commits a day, and small merges are easy. What can still
conflict:

- **The hooks above**, when upstream edits the caret drawing in `canvas.rs`. Take upstream's code
  and put the hook back.
- **Interface strings:** none in upstream's catalogs. The fork's texts are in
  `aniefork/strings.rs`; when upstream adds an interface language,
  `every_fork_text_is_translated_into_every_language` fails until it is added there.
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

- **2026-10-11 (decision, Options › Animation):** The switch for the caret's animation stays in
  the application and does not follow an operating system's animation setting: it has to work
  the same on every platform. The open item to follow Windows' setting was dropped.
- **2026-10-11 (M10, Options › Animation):** The caret's animation can be turned off: File ›
  Options has an Animation section with "Animate the caret (it glides and fades)", also the
  command `ui.caretAnimation`, saved with the interface settings. The section's three texts are
  translated into the nine interface languages in a table of the fork's own. Asked for after
  trying the calibrated caret in the app ("the animation is perfect").
- **2026-10-11 (M14, the caret calibrated against Word):** Word's caret was measured (see "What
  Word does"): the glide takes 100 ms along `cubic-bezier(0, 0, 0, 1)` whatever the distance, only
  keys and typing glide (clicks and automation don't), a second key makes the caret snap to where
  it was heading, and the blink is a pair of 100 ms fades. The first version had guessed 120 ms
  with an ease-out cubic, glided clicks and scripted moves too, set off from where the caret was
  drawn, and blinked hard; all of that now follows the measurements.
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
