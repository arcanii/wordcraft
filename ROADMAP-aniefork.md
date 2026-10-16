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
| Fork `main` is based on | upstream `86adfd3` (WordCraft 0.7.0 and the commits after it), 2026-10-16 |
| Last merge of upstream | 2026-10-16: `86adfd3`, 306 commits, no conflicts (earlier: `159b84f` on 2026-10-13, `0db810d` and `ad8c264` on 2026-10-12) |
| Fork carries | no code: only this file and `AGENTS-aniefork.md` differ from upstream |

## What the fork carries

Changes in the fork that upstream does not have. Add a row when the fork gains one, update its
last column when it is offered or merged, and delete the row once a merge of upstream brings it
back.

Nothing at present: the fork's code is upstream's.

What the fork carried, and how each ended:

| Change | In the fork | Upstream |
|---|---|---|
| **The caret glides and fades like Word's on Windows** (100 ms along Word's curve for keys and typing, a fading blink, a switch in Options) | 2026-10-11 to 2026-10-12, in `crates/ui-egui/src/aniefork/` with hooks in `lib.rs`, `canvas.rs` and the Options page (last present in `2648505`) | **Merged 2026-10-12 as [storytold/wordcraft#570](https://github.com/storytold/wordcraft/pull/570)** (`cbd9291`); upstream's is `crates/ui-egui/src/caret_glide.rs`. The fork's copy was removed at the next merge of upstream |
| **The paste-picture test passes on Windows** | 2026-10-11 to 2026-10-12, in the tests of `crates/ui-egui/src/paste_picture.rs` | Offered as [storytold/wordcraft#571](https://github.com/storytold/wordcraft/pull/571) and closed unmerged: upstream had fixed the test itself. Upstream's version was taken at the first merge |

## Open work

Investigations recorded for later, each meant to end in upstream pull requests.

| Topic | Record | State |
|---|---|---|
| **Memory use** | [`docs/memory-aniefork.md`](docs/memory-aniefork.md), with the measuring tools in `docs/memory-aniefork/` | Measured and traced on Windows on upstream `daf8caa` (2026-10-15), and on macOS on `f424471` (2026-10-16); no change upstream yet. The first fix, font fallback loading only families that have the character, is offered as [storytold/wordcraft#2345](https://github.com/storytold/wordcraft/pull/2345) (branch `font-fallback-coverage`; one emoji on macOS: 568 → 205 MB). On macOS, freed page buffers kept by the allocator show as 1.4 GB after scrolling, and PDF export copies each embedded font file. The real app is not measured yet |

## The caret: Word's glide and fade

WordCraft's caret used to jump to its new place and blink hard on and off. Since
storytold/wordcraft#570, which came from this fork, it moves and blinks the way Word's does on
Windows. This section is the record of how Word was measured and what was built from it; the code
is upstream's `crates/ui-egui/src/caret_glide.rs`.

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
- **The setting:** File › Options › Advanced › Animation › "Animate the caret (it glides and
  fades)", on by default. Off, the caret jumps and blinks on and off. It is the command
  `ui.caretAnimation` (`{"value": true|false}` sets it, no value reads it; the reply is
  `{"caretAnimation": …}`), saved in `ui.json` as `caretAnimation`. The fork's own builds of 11
  and 12 October saved it as `aniefork.caretAnimation`, which upstream's does not read: a switch
  turned off in one of those builds is on again.
- **Every platform the same.** The owner decided on 2026-10-11 that the caret's switch is in the
  application and means the same on macOS, Windows, Linux, BSD and the web, and #570 was offered
  that way. Upstream has since added a general setting above it (#791): File › Options ›
  Accessibility › Animation effects, with Use system setting (the default), On and Off. The caret
  follows that too, so under "Use system setting" a system with animation effects turned off
  stops the glide and the fades; On makes WordCraft's animations independent of the system. The
  desktop apps save both settings; the web build doesn't save any interface setting between
  visits (upstream's gap, the same for the theme and the language).
- **Other off switches:** egui's animations off (`style.animation_time == 0`), which the headless
  test harness (`egui_kittest`, so `ui_shot` too) does, so scripted screenshots never catch a
  caret in flight; and `WORDCRAFT_CARET_GLIDE_MS=0`. Any other value of that variable sets the
  glide and fade time (up to 10000), for looking at the motion slowed down.
- **The section's text** is in upstream's catalogs, in every interface language. Nine
  translations are ours and haven't been read by native speakers; the Turkish and Russian ones
  are echelon's, written when #570 was reviewed.
- **Tests:** `cargo test -p wordcraft-ui-egui caret` (Word's measured frames against the curve,
  frame-rate independence, the second-key rule, clicks and scripts, the blink's fades, view
  changes, hostile numbers, a key gliding the caret in the real canvas, and the setting as a
  command), and `advanced_animation_turns_the_caret_animation_off_and_on` in `options.rs`.

### Open
- The cases listed under "Not measured".
- A native speaker's read of our nine translations.

## Hooks in upstream files

None at present. When the fork carries code again, list every hook here (the file, where in it,
and what the hook is), so that after a merge of upstream they can be checked one by one. The last
table, for the caret, is in `git show 2648505:ROADMAP-aniefork.md`.

## Syncing with upstream

```sh
git fetch upstream
git merge upstream/main
```

Sync often: upstream lands dozens of commits a day, and small merges are easy. While the fork
carries no code a merge cannot conflict, since this file and `AGENTS-aniefork.md` are the fork's
alone. When it carries code again, what can conflict:

- **Hooks in upstream files**, when upstream edits the code around them. Take upstream's code and
  put the hook back. When upstream moves the code a hook is in (the Options page went from
  `backstage.rs` to `options.rs` in #485), the hook follows it.
- **Interface strings:** none in upstream's catalogs. The fork's texts go in a table of the
  fork's own, with a test that every text has every interface language; when upstream adds a
  language, that test fails until the language is added to the table.
- **`ATTRIBUTION.md`:** when both sides add assets (the fork has none so far).
- **A feature upstream has taken:** once a pull request made from fork work is merged upstream,
  merge upstream, take upstream's version of each hooked file
  (`git checkout upstream/main -- <file>`), delete the fork's copy, and check that
  `git diff upstream/main --stat` lists only the fork's own files.

After a merge, run `cargo xtask ci` before committing it, and update the tables above.

## Sending changes upstream

Only when the owner asks for it, one topic per pull request. The fork's version of a feature is
not what upstream gets: a pull request is the feature rewritten the way upstream would have
written it. How #570 (the caret) and #571 (the Windows test fix) were made, following the sister
fork gridcraft (storytold/gridcraft#265, #266):

- **A branch cut from `upstream/main`, in a separate worktree** (never by switching this
  checkout): `git worktree add -b <branch> <local dir> upstream/main`. A worktree on a local disk
  whose repository is on the network share needs
  `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=safe.directory GIT_CONFIG_VALUE_0=<worktree>` in the
  environment of every git command (nothing is written to the global configuration).
- **Upstream's style, no `aniefork` anywhere:** a module of its own in the crate
  (`caret_glide.rs`), settings as plain fields of `UiState`, commands in `ui_command`, interface
  text through `tl!` with an entry appended to every catalog (they must all hold the same keys),
  and upstream's documents updated as its `AGENTS.md` asks (a row in `ROADMAP.md`, a row and a
  revision line in `docs/ui-parity.md`, the command in `docs/control-protocol.md`).
- **Contributor names:** `[people.arcanii]` in `contributors/people.toml`, as its own commit (the
  same entry as in gridcraft). It landed with #570, so a branch cut from upstream's main has it.
- **The pull request:** title `Area: what changed`; body with What it does, How (and, for
  behaviour taken from Word, how it was measured), Not in this pull request, and Checks, saying
  honestly which checks ran locally. Pushed to `origin` and opened with
  `gh pr create --repo storytold/wordcraft --base main --head arcanii:<branch>`.
- **Checks on Windows:** upstream has no workflow that runs clippy (storytold/wordcraft#754 asks
  for one), so `cargo xtask ci` can stop on upstream's own code. From `2e5ecb9` to `900c0f4` it
  stopped, with Rust 1.99, at clippy's `large_enum_variant` in `crates/docx/src/read/story.rs`;
  upstream fixed that itself in storytold/wordcraft#779, and at `850fa05` the clippy step passes
  again. Before fixing such a stop, fetch upstream and look: it may be gone. When a stop is not in
  the pull request's files, run the steps one by one with only that lint allowed
  (`cargo clippy --workspace --all-targets -- -D warnings -A clippy::<the lint>`,
  `cargo test --workspace`, `cargo xtask assets`, `layers`, `wasm`) and say so in the pull
  request. Run the ignored translation audit
  (`cargo test -p wordcraft-ui-egui untranslated_interface_text -- --ignored --nocapture`) when
  interface text was added, and read what it lists: on `2e5ecb9` it reports two of upstream's
  Mailings texts. (`paste_inserts_clipboard_pixels`, which used to fail on Windows, passes on
  upstream since 2026-10-11.)
- **After it is open:** a maintainer may push to the branch (echelon merged main into #570 and
  added the strings for two new catalogs). Fetch `origin <branch>` and fast-forward before
  touching it. When main moves and the pull request conflicts, merge `upstream/main` into the
  branch and push; never rebase or force-push. A new catalog upstream means the feature's strings
  are missing from it: add them in the same merge.
- **Rows in upstream's log tables:** upstream adds each new row at the top of `ROADMAP.md`'s
  progress log and of the revision history in `docs/ui-parity.md`. A pull request whose row is
  also at the top conflicts with every commit upstream lands. Put the row three or four rows
  down, among rows of the same date; rows added above it then merge cleanly (#570 stopped
  conflicting there once its rows were moved). The same goes for `contributors/people.toml`,
  where new entries land at or near the end: put the entry among the older ones. And for the
  catalogs (`i18n/*.tsv`): every branch appends its strings at the end, so strings appended
  there conflict with each branch of the same days that lands. Put a pull request's strings
  next to related older ones (#570's follow "Show rulers"); their order in a catalog is free.

## Building

The workspace says `rust-version = "1.90"`, but egui 0.36 needs Rust 1.95 or newer; an older
toolchain fails with a dependency error (pdfcraft's fork found the same and carries a fix).
`cargo xtask ci` also needs the `wasm32-unknown-unknown` target.

Give every checkout its own `CARGO_TARGET_DIR`: the fork and a worktree made for an upstream pull
request must not share one. Cargo names the workspace's artifacts by their path inside the
workspace, so a second checkout finds the first one's artifacts "fresh", and `xtask` keeps the
root of the checkout it was compiled in, so `cargo xtask ci` then checks the other checkout
without saying so. After a gate run, look in its log for the path in `Compiling xtask … (path)`
and for the fork's own tests (`aniefork::`).

Upstream runs its test suite on FreeBSD only (`.github/workflows/freebsd.yml`), so a test that
fails on Windows goes unnoticed there. The fork is developed on Windows: run the whole gate, and
don't set `WGPU_BACKEND` for it (a graphics test reads eframe's default backends, which follow it).

## Log

Newest first.

- **2026-10-16 (font fallback offered upstream):** Pushed `font-fallback-coverage` to `origin`
  and opened [storytold/wordcraft#2345](https://github.com/storytold/wordcraft/pull/2345) against upstream's
  main, with What it does, How (with the before/after measurements), Not in this pull request
  and Checks. Upstream's main had not moved, and the branch merges cleanly. It overlaps open
  #1972, which adds a check inside `file_face_names`: whichever lands second moves that check
  onto `TableReader`, as the pull request says.
- **2026-10-16 (font fallback fix, on a branch):** The first item of the memory plan, as an
  upstream-ready branch `font-fallback-coverage` cut from `upstream/main` (`86adfd3`) in the
  worktree `../wordcraft-font-fallback`, with upstream's progress rows. The system fallback asks
  the coverage index before loading a preferred family. The index now reads character maps
  only, through a table reader shared with the font folder scan, so it covers files of any
  size. Measured with `measure_macos.py`: one emoji 568 → 205 MB (engine), 642 → 300 MB
  (interface); Latin and Chinese unchanged. Three new tests, one of which fails without the
  check. `cargo xtask ci` passes on the branch (2,772 tests). Not pushed: offering it upstream
  is the owner's call.
- **2026-10-16 (memory use measured on macOS):** Asked whether the Windows measurements hold on
  macOS. Measured the merged fork headless (Metal, real GPU) and through the CLI with a new
  sampler, `docs/memory-aniefork/measure_macos.py`, which reads the kernel's peak footprint.
  The causes are the same; three cost more or show up only here: font fallback (+528 MB for one
  emoji, against +141 MB on Windows), large page buffers that the macOS allocator keeps after
  they are freed (1.4 GB after paging, 134 MB with its cache off), and PDF export copying each
  embedded font file (+183 MB for Apple Color Emoji). Also found: every `ui_shot` picture costs
  about 450 MB for a moment, so headless peaks are the harness's. Recorded in
  `docs/memory-aniefork.md`. Nothing changed in the code.
- **2026-10-16 (fourth merge of upstream):** Merged upstream `86adfd3`: 306 commits, with the
  release WordCraft 0.7.0 among them and the shared PDF writer (`craft-pdf`) adopted. No
  conflicts, as the fork carries no code. `cargo xtask ci` passes on the fork's tree (2,769
  tests), this time on macOS with rustup's Rust 1.98.1 (Homebrew's Rust has no wasm target).
- **2026-10-15 (memory use investigated):** Asked why WordCraft uses so much memory and how to
  reduce it, possibly as an option. Measured upstream `daf8caa` (0.7.0) headless and through the
  CLI, and traced the code. Findings, strategies and an option design are in
  `docs/memory-aniefork.md`; the tools that measured them are in `docs/memory-aniefork/`. Nothing
  changed in the code. A measuring pitfall found on the way: `ui_shot` scripts must render after
  each step, or pending GPU work shows gigabytes the app never uses.
- **2026-10-14 (the README and upstream's marks stay):** Asked whether upstream's rules for
  forks need the README changed, as the sister fork of gridcraft did the same day. No:
  upstream's brand licence says a fork that distributes or publishes a modified version must
  remove the ArtCraft marks, and that "a fork kept only to propose changes back to this
  repository may keep the marks unchanged". This fork is the second kind (the owner: "This
  fork was for developing features for upstream"): its code is upstream's, it has no releases
  or tags, and its release workflow runs only on a `release` branch or by hand. Written into
  `AGENTS-aniefork.md`, with what would change the answer.
- **2026-10-13 (third merge of upstream):** Merged upstream `159b84f`: 119 commits, with the
  release WordCraft 0.6.1 among them, the first release that has the caret's glide and fade.
  No conflicts, as the fork carries no code. `cargo xtask ci` passes on the fork's tree.
- **2026-10-12 (the fork's caret copy removed; the fork carries no code):** Merged upstream
  `ad8c264` (49 commits, #570's merge among them) and removed the fork's own caret:
  `crates/ui-egui/src/aniefork/` deleted, and `lib.rs`, `canvas.rs` and `options.rs` taken from
  upstream, so the hooks are gone. Only this file and `AGENTS-aniefork.md` now differ from
  upstream. Before deleting, the fork's `caret.rs` was compared with upstream's
  `caret_glide.rs`: the same code, and upstream's also follows its new Animation effects
  setting (#791, see "Every platform the same" above). The saved setting's key changed from
  `aniefork.caretAnimation` to `caretAnimation`. The worktrees and local branches of #570 and
  #571 were removed; both branches remain on `origin`. `cargo xtask ci` passes on the fork's
  tree (1,523 tests).
- **2026-10-12 (#570 merged upstream):** echelon merged
  [storytold/wordcraft#570](https://github.com/storytold/wordcraft/pull/570) into upstream's
  main (`cbd9291`) after one more merge of main on the branch. It landed as offered: the caret
  glides and fades by default, File › Options › Advanced › Animation turns it off. The fork's
  own copy (`aniefork/`) is still in place until upstream is merged again.
- **2026-10-12 (#570: a fifth merge of main):** All eleven catalogs conflicted: upstream's
  new strings landed where #570's three were. Merged as `5dc8fe9`, with #570's strings moved
  next to "Show rulers". Upstream's caret work for complex scripts changed `canvas.rs` around
  the hooks and merged cleanly. Checked with the worktree's own build cache: fmt, clippy, the
  interface crate's 232 tests, assets, layers and wasm; the translation audit still lists only
  upstream's two Mailings texts.
- **2026-10-12 (#570: two more merges of main):** Both conflicts were in
  `contributors/people.toml`, where upstream added a contributor in the place #570's entry was
  (`49c9408`, then `710d450`, which moved the entry up among the older ones). The log tables
  merged cleanly both times. Checked with a build cache of the worktree's own: fmt, clippy with
  nothing allowed, the interface crate's tests, assets, layers and wasm.
- **2026-10-12 (first merge of upstream):** Merged upstream `0db810d` into the fork's `main`:
  77 commits, among them File › Options as panes, Outline view, vertical text for sections, the
  Customize dialogs and the Turkish and Russian interface languages. Three files conflicted.
  `lib.rs`: the fork's hooks and upstream's new modules, settings and command sat side by side;
  both kept, the fork's setting and its last arm still last. `backstage.rs`: upstream moved the
  Options page to `options.rs`, so its version is taken and the fork's hook moved to the
  Advanced pane, between Display and Customize, where #570 has it. `paste_picture.rs`:
  upstream's own fix of the Windows test taken; the fork no longer carries that change. In the
  fork's files: the section is spaced like the pane's own, its test opens the Advanced pane,
  and the three texts gained Turkish and Russian (echelon's, from #570). `cargo xtask ci`
  passes.
- **2026-10-12 (the DOCX reader's clippy stop: fixed upstream, nothing to send):** The
  `large_enum_variant` stop met while merging main into #570 was to be offered upstream as a fix
  of its own. A fetch showed upstream had boxed the variant itself the same day
  (storytold/wordcraft#779). Checked at upstream `850fa05` with Rust 1.99:
  `cargo clippy --workspace --all-targets -- -D warnings` passes. No branch or pull request made.
  #570's branch does not hold that fix yet (its last merge of main is `900c0f4`); it arrives with
  the next merge of main.
- **2026-10-12 (#570 merged with main again; #571 closed):** Upstream's main moved 30 commits and
  #570 conflicted in 15 files. Before that, echelon (a maintainer) had merged main into the
  branch once (`9684f9b`) and added the three strings to the new Turkish and Russian catalogs,
  leaving the default (animation on) to the maintainer. This merge (`75b991f`): upstream rebuilt
  File › Options as panes in `options.rs` (#485), so the Animation section and its test moved from
  `backstage.rs` to the Advanced pane, between Display and Customize; the eleven catalogs,
  `lib.rs` and the two documents kept both sides. Locally: fmt, clippy with one upstream lint
  allowed (see "Checks on Windows"), 1,311 tests, assets, layers and wasm pass; the Advanced pane
  looked at in English and Turkish with `ui_shot`. #571 was closed without merging: main already
  had its own fix for the test. Later the same day main moved three more commits and #570
  conflicted again, in `ROADMAP.md` only (both sides had added a row at the top of the progress
  log). Merged twice more (`008ac61`, `5eb7dab`), with #570's rows in the two log tables moved
  three rows down; the second of those merges had no conflicts. Locally each time: fmt, clippy
  with the one upstream lint allowed, the interface crate's 212 tests, assets, layers and wasm.
- **2026-10-11 (the Windows test fix offered upstream):** Pull request
  [storytold/wordcraft#571](https://github.com/storytold/wordcraft/pull/571), from the branch
  `paste-picture-test-windows` cut from upstream `1d4f741`: the same change to
  `paste_inserts_clipboard_pixels` the fork carries, alone. Locally: fmt and the crate's 159 tests
  pass. #570's description points to it.
- **2026-10-11 (the caret offered upstream):** Pull request
  [storytold/wordcraft#570](https://github.com/storytold/wordcraft/pull/570), from the branch
  `caret-glide` cut from upstream `1d4f741`: the caret's glide and fading blink and the Options ›
  Animation switch, rewritten in upstream's style (`caret_glide.rs`, a plain `caret_animation`
  setting, entries in the nine catalogs, upstream's documents), with the measurement method and
  numbers in its description. Locally: fmt, clippy, 1,097 tests, the translation audit, assets,
  layers and wasm pass; the one test that fails on Windows is upstream's own. The fork keeps its
  `aniefork` version until upstream's lands.
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
