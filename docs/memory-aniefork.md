# Memory use: findings (aniefork)

Status: investigated 2026-10-15 on Windows, on upstream `daf8caa` (WordCraft 0.7.0); measured on
macOS 2026-10-16, on the fork's `f424471` (upstream `86adfd3`, 50 commits later), headless and
then in the real app. The first fix is offered upstream (font fallback, "What to do" item 1); the
rest is the record to work from. Changes made from it go upstream as pull requests, as
[`AGENTS-aniefork.md`](../AGENTS-aniefork.md) says.

## Summary

- **The engine is small.** Without the interface, a 74-page text document takes about 35 MB.
- **The interface starts large.** About 380 MB with an empty document in the headless measurement.
  Much of that is graphics overhead, and on a software GPU it includes what a real GPU keeps in
  its own memory. **The real app has not been measured yet.**
- **Four things add the most on top:**
  - **Font fallback:** one Chinese word adds about 66 MB, one emoji about 140 MB. Whole font
    families are loaded before WordCraft checks whether they contain the character, and they are
    never released.
  - **Zoom:** whole-page images are kept for about 3 screens around the view and grow with the
    square of the zoom. At 200%, memory is about 360 MB higher.
  - **Pictures:** decoded at full resolution into caches of 128 MB each that empty completely
    when full. PDF export of a picture-heavy document peaks at 568 MB.
  - **Undo:** up to 500 full copies of the document, with no limit by size (not measured).
- **Scrolling does not pile up memory.** Page images out of range are released. An earlier
  measurement that seemed to show gigabytes was a fault in how the measuring tool was used (see
  "A measuring pitfall").
- **On macOS the same causes cost more** (see "Measurements on macOS"):
  - **Font fallback is about four times worse:** one emoji keeps about 530 MB, because every
    CJK family listed before Apple Color Emoji is installed on a Mac and large. One Chinese word
    keeps about 80 MB.
  - **Headless, scrolling looked as if it piled up memory:** 1.4 GB after paging through 74
    pages, most of it freed blocks macOS's allocator keeps. The real app does not do this: it
    keeps 26 MB of freed blocks after the same paging. The harness's screenshots were the cause.
  - **PDF export copies each embedded font file whole:** another 75 MB for a Chinese word and
    183 MB for an emoji. This is the case on every platform.
- **The real app on macOS** (see "The real app on macOS"):
  - **588 MB with a 74-page text document open, but WordCraft's own share is about 100 MB.**
    Any Metal window costs about 295 MB of GPU memory on this Mac, even a native Swift one that
    only clears the screen. A blank eframe window costs about 490 MB.
  - **Pictures are the largest cost of WordCraft's own:** the picture document takes 277 MB more
    than the text one. 206 MB of that is blocks already freed but still counted, left by opening
    the file: it is there before any picture is drawn.

## Measurements

**Setup:**
- Windows 11, with release builds of upstream `daf8caa`.
- Private bytes (Task Manager's "Memory") sampled every 50 ms with
  [`measure.ps1`](memory-aniefork/measure.ps1).
- The interface was run headless with the `ui_shot` example: a 1440×900 window at 2× pixel
  density, on the software adapter (`WGPU_BACKEND=dx12`).
- The engine was run alone through `wordcraft-cli`.
- The test documents are synthetic, made by [`make_docs.py`](memory-aniefork/make_docs.py):
  - `text-100p`: 74 pages of text;
  - `pics-100p`: 89 pages with 20 PNG pictures of 2400×1800, a 92 MB file;
  - four one-line documents for font fallback.

**Interface (headless)**, with a picture rendered after every step, as the app renders every frame:

| Situation | Private memory |
|---|---|
| Empty document | about 380 MB |
| The sample document | about 385 MB |
| `text-100p` open | about 410 MB |
| `text-100p`, paging through all 74 pages | about 520 MB, flat |
| `text-100p` at 200% zoom, paging through 30 pages | about 900 MB, flat |
| `pics-100p`, paging through all pages | 620–715 MB, rising and dropping as the picture cache fills and empties |
| One-line document: Latin only | peak 495 MB |
| One-line document: a Chinese word | peak 561 MB (+66) |
| One-line document: an emoji | peak 636 MB (+141) |
| One-line document: Chinese, emoji, Thai, Devanagari, Arabic | peak 629 MB (+134) |

**Engine only (CLI):**

| Situation | Private memory |
|---|---|
| `info text-100p` (open and lay out) | 32 MB |
| `convert text-100p` to PDF | 35 MB |
| `info pics-100p` | 129 MB |
| `convert pics-100p` to PDF | peak 568 MB, 199 MB at the end |

**Caveats:**
- The software adapter keeps GPU memory in ordinary RAM, so the real app on a graphics card will
  differ, most of all in the baseline and in the page images.
- CLI runs shorter than about 0.3 s can miss their peak with 50 ms sampling, so the two `info`
  rows are lower bounds. The one-line documents run through the CLI were too quick to measure
  and are left out.

### A measuring pitfall

egui_kittest hands work to the GPU only when it renders a picture. A `ui_shot` script that runs
only `{"steps": n}` therefore keeps every page-image upload and release pending. Paging through
`text-100p` that way showed 2.4 GB, and `pics-100p` 3.1 GB, which the app never uses. Render
after each step (`{"shot": …}`), as the scripts from
[`make_ui_scripts.py`](memory-aniefork/make_ui_scripts.py) do.

Rendering a picture has a cost of its own. On macOS each `{"shot": …}` adds about 410–450 MB for
a moment, even for an empty document, and the memory goes back afterwards: with no shot, the
empty document stays at 64 MB. The peaks of the headless runs are therefore the harness's, not
the app's; compare the settled values between shots. The Windows peaks above probably include
the same cost, which was not checked there. The cause inside `egui_kittest` was not traced.

## Measurements on macOS

**Setup:**
- MacBook Pro, Apple M3 Max with 64 GB, macOS 27.0.1, with release builds of the fork's
  `f424471` (upstream `86adfd3`, 50 commits after `daf8caa`).
- Measured with [`measure_macos.py`](memory-aniefork/measure_macos.py). It reports the physical
  footprint, which is what Activity Monitor calls "Memory". On Apple silicon the footprint
  includes the process's GPU memory, as the Windows software adapter's did.
- Its peak is the kernel's own lifetime maximum, so short CLI runs are measured exactly.
- The same documents and `ui_shot` scripts as on Windows. `ui_shot` used the Metal backend on
  the real GPU, with no `WGPU_BACKEND`.

**Interface (headless).** "Settled" is the footprint during the 4 s hold at the end of each
script, between pictures. "No allocator cache" is the same run with `MallocLargeCache=0`, which
makes macOS give freed large blocks back at once:

| Situation | Settled | Settled, no allocator cache | Peak (with the picture cost) |
|---|---|---|---|
| Empty document | 110 MB | 58 MB | 579 MB |
| The sample document | 129 MB | | 596 MB |
| `text-100p` open | 155 MB | | 623 MB |
| `text-100p`, paging through all 74 pages | 1,413 MB | 134 MB | 1,890 MB (630 MB without the cache) |
| `text-100p` at 200% zoom, paging through 30 pages | 1,033 MB | 239 MB | 1,536 MB (803 MB) |
| `pics-100p`, paging through all pages | 1,547 MB | 260 MB | 2,102 MB (827 MB) |
| One-line document: Latin only | 115 MB | | 583 MB |
| One-line document: a Chinese word | 194 MB (+79) | | 663 MB |
| One-line document: an emoji | 643 MB (+528) | | 1,114 MB |
| One-line document: Chinese, emoji, Thai, Devanagari, Arabic | 642 MB (+527) | | 1,115 MB |

The one-line documents were run twice, and the runs agreed within 2 MB.

**Engine only (CLI):**

| Situation | Peak |
|---|---|
| `info text-100p` (open and lay out) | 47 MB |
| `convert text-100p` to PDF | 45 MB |
| `info pics-100p` | 321 MB |
| `convert pics-100p` to PDF | 556 MB |
| `info` of the one-line documents: Latin, Chinese, emoji, all scripts | 20, 97 (+77), 566 (+546), 568 MB |
| `convert` of the same to PDF | 17, 169, 748, 824 MB |
| `render` of the first three to PNG | 28, 105, 575 MB |

**What is different on macOS:**

- **Freed memory stays counted, in the harness.** After paging through `text-100p`,
  `vmmap --summary` shows a footprint of 1.4 GB. Of that, 1.2 GB is "Malloc Large (empty)": 247
  large blocks freed and kept, still dirty, by macOS's allocator for reuse. The process's own use
  is about 61 MB of small blocks, 24 MB of large ones and 61 MB of GPU textures.
  - macOS gives such memory back under memory pressure, so it is not a leak. But Activity
    Monitor counts it until then.
  - **Most of it was the harness's.** Each screenshot allocates about 450 MB for a moment, and
    the paging script takes 95 of them. The real app, paging the same way, keeps only 26 MB of
    freed blocks (see "The real app on macOS").
- **The fallback list is long on a Mac.** For a character no loaded font has, `system_fallback`
  goes through `SYSTEM_FALLBACKS` (`crates/fonts/src/fontdb.rs:504`) and loads every installed
  family before the one that covers it. A Mac has nearly all the CJK families on that list.
  File sizes on this Mac (est. from the files; the measured growth agrees):
  - a Chinese word loads Helvetica Neue (4 MB), Arial (3 MB) and PingFang SC (the whole
    PingFang collection, 75 MB), about 82 MB;
  - an emoji also loads Hiragino Sans (W0–W9, 56 MB), Apple SD Gothic Neo (53 MB), Heiti SC
    (STHeiti Light and Medium, 106 MB), STHeiti (13 MB), Arial Unicode MS (22 MB), Apple
    Symbols (1 MB) and finally Apple Color Emoji (183 MB), about 516 MB in all.
- **PDF export copies each font file again.** `craft_pdf::write::font` (storytold/craft-libs
  `crates/pdf/src/write.rs:149`) copies the whole file of every face it embeds before handing it
  to Krilla: `convert` adds 72 MB over `info` for the Chinese word (PingFang) and 182 MB for the
  emoji (Apple Color Emoji). It is the same on Windows, where the files are smaller.

## The real app on macOS

**Setup:**
- The same Mac, with a release build of the desktop app from upstream `86adfd3`'s code.
- A profile of its own (`WORDCRAFT_PROFILE_DIR`), so the runs leave the owner's settings alone.
- A 1440 × 900 window at 2×, opened on the test document, then driven over the control channel
  by [`drive_app.py`](memory-aniefork/drive_app.py): page down, hold, `vmmap --summary`, quit.
- Measured with `measure_macos.py`, as for the headless runs.

**WordCraft:**

| Situation | Footprint | GPU memory | Freed large blocks kept |
|---|---|---|---|
| `text-100p` open | 588 MB | 424 MB | 26 MB |
| `text-100p`, after paging through all 74 pages | 650 MB | 473 MB | 26 MB |
| The same, with `MallocLargeCache=0` | 597 MB | 473 MB | none |
| `pics-100p` open | 865 MB | 424 MB | 206 MB |
| `pics-100p`, after paging through all pages | 876 MB (peak 1,071 MB) | 473 MB | 125 MB |

"GPU memory" is vmmap's "owned unmapped (graphics)": private Metal allocations.

**Controls, to see whose the GPU memory is:**

| Program | Footprint |
|---|---|
| A native Metal window in Swift that only clears the screen (no Rust, no wgpu) | 295 MB, steady |
| A plain wgpu window that only clears the screen (no egui) | 458 MB at first, then 298 MB |
| A blank eframe window, WordCraft's size and eframe features | about 490 MB, sometimes released to 43–79 MB after a few seconds |
| A wgpu device with no window | 6 MB |
| Dock, Terminal, Finder (private GPU memory only) | 1–3 MB each |

**What this shows:**
- **Most of the baseline isn't WordCraft's.** On this Mac (macOS 27.0.1, M3 Max), any Metal
  window costs about 295 MB of GPU memory. eframe's window costs about 190 MB more than a plain
  wgpu one; why was not traced. With the text document open, WordCraft's own share is about
  100 MB.
- **The window's own textures are small:** 15 MB with the document open, 54 MB after paging.
  - A page image is 13 MB at 2×.
  - The glyph sheet is 8192 × 32, 1 MB.
  - The largest texture side is 8192, as in the harness.

  This was listed with a temporary texture list added to `ui.inspect`, not committed.
- **The headless gigabyte after paging was the harness's.** The real app keeps 26 MB of freed
  blocks after paging, so reusing page buffers ("What to do", item 3) is worth about 50 MB.
- **Pictures are WordCraft's largest cost of its own.** The picture document takes 277 MB more
  than the text document once open:
  - 97 MB of large blocks in use, among them the 92 MB of encoded pictures;
  - 206 MB of freed blocks still counted. **These come from opening the file, not from
    decoding** ("What to do", item 8): the first page has no picture, so none had been drawn
    yet. This was first read
    as decode buffers; the pictures branch ("What to do", item 2) showed otherwise.

## Where the memory goes

From a read-only survey of upstream `daf8caa`. The key references were checked by hand. Line
numbers will drift as upstream moves. Sizes marked "est." are estimates, not measurements.

### 1. Fixed graphics overhead

- **GPU allocator:** wgpu's default `MemoryHints::Performance` makes the DX12 allocator take
  blocks of at least 128 MB of device memory and 64 MB of upload memory, the latter in system
  RAM. It keeps the last block (wgpu-hal 30.0.1 `lib.rs:445-450`; gpu-allocator 0.28 `d3d12`).
- **Glyph sheet:** egui's font atlas is as wide as the maximum texture side (8192) and doubles in
  height until it is 80% full. It can reach about 214 MB before it resets. A CPU copy is kept,
  and each growth copies it (epaint 0.36.2 `text/fonts.rs:730`, `texture_atlas.rs`). About
  8–16 MB for a Latin interface, more for CJK (est.).

### 2. Page images

- **One texture per page:** each page is a single texture, not tiles (`crates/ui-egui/src/canvas.rs:35`).
- **Resolution:** zoom × 96/72 × pixels per point. Each side is capped at the maximum texture
  side (`canvas.rs:1011`).
- **What is kept:** textures for pages within one screen width and 3 screen heights of the view
  (`TEXTURE_KEEP_SCREENS`, `canvas.rs:32`, used at `1141`). Others are released.
- **Size per Letter page:**

  | Zoom | Pixels per point | Size |
  |---|---|---|
  | 100% | 1.0 | 3.4 MB |
  | 100% | 1.5 | 7.8 MB |
  | 200% | 1.5 | 31 MB |

  The measured cost is about 110 MB at 100% and about 360 MB more at 200% (headless).
- **Extra copies per render:** each render allocates the page's pixels, copies them into an
  egui image, and wgpu copies them again for upload (`crates/render/src/lib.rs:489-519`).
- **Rasteriser rebuilt per page:** each page render builds a new rasteriser context with a pool
  of up to 8 threads (`render/src/lib.rs:499`), which costs time as well as memory.

### 3. Fonts

**Whole families, kept forever:**
- `load_cataloged` reads every file of a family whole with `std::fs::read`
  (`crates/fonts/src/fontdb.rs:1179-1199`).
- Faces are never unloaded (`fontdb.rs:87-111`).
- There is no memory-mapping; the workspace forbids `unsafe`.

**Fallback loads before it checks:**
- `system_fallback` (`fontdb.rs:1579`) goes through the script's fallbacks, then the preferred
  families, then `SYSTEM_FALLBACKS`. It loads each installed family first and checks coverage
  after, so families that don't cover the character stay loaded.
- On Windows (file sizes from `C:\Windows\Fonts`):
  - a CJK character loads Arial, Segoe UI and Microsoft YaHei, about 60 MB;
  - an emoji also loads Yu Gothic, Malgun Gothic, Segoe UI Symbol and Segoe UI Emoji, about
    150 MB.
- This matches the measured +66 and +141 MB.

**Smaller items:**
- **Outline cache:** 50,000 glyph outlines, emptied completely when full (`fontdb.rs:574`).
- **CJK interface font:** once a CJK interface language, the language picker or a CJK file name
  asks for it, one system CJK font of up to 128 MB is kept for good (`crates/ui-egui/src/theme.rs:499-517`).
- **Caption fallback faces:** copied twice, once by us and once by epaint (`theme.rs:406`).

### 4. Pictures

- **Encoded bytes:** kept for the whole session and never pruned when a picture is deleted
  (`crates/doc/src/lib.rs:681`, `1266-1278`).
- **Decoding:** at full source resolution (`render/src/lib.rs:173-208`), into two caches,
  `images()` and `vectors()`. Each holds 128 MB and 256 entries, and empties completely when
  full (`render/src/lib.rs:112-168`). That emptying is the rise and drop measured with `pics-100p`.
- **Size limit applied too late:** picture adjustments decode the whole picture before checking
  the 80-megapixel limit (upstream issue #1321; `crates/engine/src/cmd/objects.rs:1038-1046`).

### 5. Undo

- **Full copies:** every step stores a whole `Document`, up to `MAX_UNDO` = 500 steps
  (`crates/engine/src/session.rs:571`, `845-872`), with no limit by size.
- **What is shared and what isn't:** paragraphs and pictures are shared between steps, but
  styles, numbering, comments, revisions and settings are copied each time.
- **Versions:** keeps up to 50 more full copies (`crates/engine/src/cmd/tools.rs:396-401`).
- **Not measured:** about 50–500 KB per step for a document that came from Word (est.).

### 6. Smaller items

- **Layout cache:** keeps up to 4× its live entries as stale ones (`crates/layout/src/lib.rs:2307-2313`),
  and the spare capacity of growing lists (upstream PR #1753).
- **Recovery:** every 10 s while there are unsaved changes, it copies the document, writes it as
  JSON with pictures as number arrays, and reads it back to compare
  (`crates/engine/src/recovery/desktop.rs:244-253`, `462-490`).
- **AutoSave:** builds the whole `.docx` in memory before writing it.
- **Proofing:** about 4–5 MB for English.

## What to do

### No downside: just do them (upstream pull requests)

1. **Font fallback.** **Offered upstream as [storytold/wordcraft#2345](https://github.com/storytold/wordcraft/pull/2345)**, from the branch
   `font-fallback-coverage` (`8c7bc50`, cut from upstream `86adfd3`).
   - **What changed:** the coverage index is asked before a preferred family loads, and a family
     none of whose files has the character is skipped. The index now reads only each file's
     character maps, so it also covers files over 40 MB, such as PingFang and Apple Color Emoji.
   - **What didn't:** a family that has the character still loads whole, with all its styles.
     Upstream relies on families loading complete, since a partial family resolves the wrong
     face and caches it. The fallback order is unchanged.
   - **Measured on macOS, before → after:**

     | One-line document | Engine (`info`) | PDF (`convert`) | Interface (settled) |
     |---|---|---|---|
     | Emoji | 568 → 205 MB | 748 → 385 MB | 642 → 300 MB |
     | All five scripts | 568 → 282 MB | 824 → 540 MB | 642 → 379 MB |
     | Latin, or a Chinese word | unchanged | unchanged | unchanged |

   - **Not measured on Windows.** Expected there: an emoji loads only Segoe UI Emoji.
2. **Pictures.** Decode at the size shown on screen. Drop the least-recently-used entries by
   bytes instead of emptying the cache. Prune media no step of the undo history uses. Check
   limits before decoding (#1321).
   - **Done on the local branch `picture-decode-size`** (`be2d0b1`, from upstream `eae2421`),
     the first two parts; not offered upstream yet. A picture is decoded halved as often as its
     drawn size allows (box-averaged, premultiplied), and the cache drops its least recently
     used entries. `cargo xtask ci` passes on it.
   - **Measured in the real app on macOS,** paging through `pics-100p`, two runs each:

     | Zoom | Peak, before → after | Settled, before → after |
     |---|---|---|
     | 100% | 1,069 → 933–936 MB | 875 → 919–923 MB |
     | 50% | 1,078 → 968–970 MB | 875–890 → 944–947 MB |

     The text document is unchanged (645 → 643 MB).
   - **Peaks fall by 110–140 MB, but the settled footprint doesn't.** The decoded pictures are
     4× smaller at 100% and 16× at 50% on a 2× screen. But the cache now stays full instead of
     emptying and refilling, about 30 MB more live data. The settled number is dominated by
     freed blocks the allocator keeps: 125–305 MB across runs of the same build.
   - **The bigger target was opening the file.** It left 206 MB of freed blocks before any
     picture is drawn; see item 8.
3. **Rasteriser.** Reuse one rasteriser context and thread pool across page renders, and reuse
   the page's pixel buffers instead of allocating new ones for each render. This is faster as
   well as smaller. In the real app on macOS it saves only about 50 MB of memory: the allocator
   keeps 26 MB of freed blocks after paging, and the app takes 597 MB with that cache off
   against 650 MB with it. The gigabyte measured headless was the harness's.
4. **Undo.** Share unchanged styles, numbering, comments and revisions between steps (behind
   `Arc`), and add a size cap next to the step cap.
5. **Layout.** Release spare capacity and keep fewer stale entries (#1753 is upstream's start).
6. **Recovery and AutoSave.** Write straight to the file instead of building copies, and compare
   a hash instead of reading the snapshot back.
7. **PDF export.** Hand Krilla the font database's shared bytes instead of a copy of each font
   file. This is a change to `craft_pdf::write::font` in storytold/craft-libs, which takes a
   `&[u8]` today, and to its caller in `crates/pdf/src/lib.rs`. Expected: the size of every
   embedded font file less at each export (75 MB for PingFang, 183 MB for Apple Color Emoji).
8. **Opening a DOCX.** Traced 2026-10-16 from the 206 MB of freed blocks the picture document
   leaves once open. Opening goes in three steps:
   - `io.rs` reads the whole file into memory (`std::fs::read`): 92 MB here.
   - `Package::open` unzips every part into its own buffer: another 92 MB for the pictures.
   - `Reader::media_for` (`crates/docx/src/read/mod.rs`) copies each picture into the document
     (`Arc::new(bytes.to_vec())`), and so do the macro and embedded-object parts kept for saving.

   The file and the package's copies are freed once the document is read: about 184 MB, plus
   the parsed XML.
   - **Done on the local branch `docx-open-shares-parts`** (`eb49c56`, from upstream `eae2421`),
     with upstream's progress rows; not offered upstream yet. The package holds its parts as
     `Arc<Vec<u8>>`, the type the document keeps pictures in, and hands them over. One new test
     (the document's picture is the package's own bytes); `cargo xtask ci` passes.
   - **Measured in the real app, two runs each:** once open, 862–866 → 772 MB; freed blocks
     kept 206 → 118 MB; peak 915–932 → 821–823 MB. `wordcraft-cli info pics-100p` peaks at
     319 → 231 MB. The text document is unchanged.
   - **What's left** is mostly the whole file held in memory while it is unzipped, 92 MB here.
     Unzipping straight from the file (`zip` reads any `Read + Seek`) would remove it. That
     needs a file-based open path beside the bytes-based one in `io.rs`, so it is left for a
     separate change.

### Trade-offs: candidates for an option

A setting such as File › Options › Advanced › "Use less memory", the same on every platform as the
owner wants settings to be, could switch these together:

| Change | Cost |
|---|---|
| Keep fewer page images ready (1 screen instead of 3), or a byte budget; tiles at high zoom | pages redraw when scrolling back |
| Smaller picture cache | pictures are decoded again more often |
| `MemoryHints::MemoryUsage` for wgpu (smaller blocks; DX12 and Vulkan only, Metal ignores it) | slightly slower allocation; needs a restart |
| Cap the glyph sheet (a lower maximum texture side) | glyphs are drawn again more often; also caps page images unless they are tiled |

## Upstream context

| Item | State | What it is |
|---|---|---|
| #1753 | open | layout uses less memory on long tables |
| #1321 | open | image limits applied after allocation |
| #1354 | open | font family caches keep arbitrary keys |
| #1595 | open | language packs downloaded on demand |
| #848 | fixed by #874 | memory grew with every font change |
| #1149 | merged | equation pictures share a few atlas textures |

## Still open

- **The real app on Windows.** It needs a WordCraft window on the desktop, so ask the owner
  first (the GPU caution in the build notes). On macOS it was measured on 2026-10-16, with the
  owner's OK.
- **eframe's 190 MB over a plain wgpu window** on macOS, and why it is sometimes released after
  a few seconds: not traced. It would be a change in egui, not WordCraft.
- **Undo** over a long editing session.
- **Linux:** not measured.
- **Craft-fonts in a release build:** their resident size is not measured.

## Measuring again

1. Build release copies of upstream's `main` in a separate worktree with its own
   `CARGO_TARGET_DIR` (never shared with the fork's checkout; see `ROADMAP-aniefork.md` ›
   Building):
   `cargo build --release -p wordcraft -p wordcraft-cli` and
   `cargo build --release -p wordcraft-ui-egui --example ui_shot`.
2. Make the documents and scripts:
   `python -I docs/memory-aniefork/make_docs.py <dir>`, then
   `python -I docs/memory-aniefork/make_ui_scripts.py <dir>`.
3. Run each script through the sampler, in PowerShell:

   ```powershell
   docs/memory-aniefork/measure.ps1 -Label "text-100p" -Exe <ui_shot.exe> -ArgLine "<dir>/ui-text-100p.jsonl" -Env @{ WGPU_BACKEND = "dx12" }
   ```

   The CLI works the same way, without `-Env`, for example with
   `-ArgLine "convert <dir>/pics-100p.docx <dir>/out.pdf"`.

   On macOS, use `measure_macos.py` instead, with a label, any environment variables, `--`
   and the command:

   ```sh
   python3 -I docs/memory-aniefork/measure_macos.py "text-100p" -- <ui_shot> <dir>/ui-text-100p.jsonl
   python3 -I docs/memory-aniefork/measure_macos.py "text-100p, no cache" MallocLargeCache=0 -- <ui_shot> <dir>/ui-text-100p.jsonl
   ```

   `vmmap --summary <pid>` during a script's final hold shows where the footprint is.
4. **The real app** (it opens a window, so ask the owner first). Build it with
   `cargo build --release -p wordcraft`. Start it under the sampler with a profile of its own,
   then drive it from a second command:

   ```sh
   python3 -I docs/memory-aniefork/measure_macos.py "app text-100p" WORDCRAFT_PROFILE_DIR=<profile> -- <wordcraft> --control 7987 <dir>/text-100p.docx &
   python3 -I docs/memory-aniefork/drive_app.py 7987 <profile> --pages 95 --vmmap 1
   ```
