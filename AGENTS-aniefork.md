# aniefork: this checkout is a fork

This repository is arcanii/wordcraft, a fork of storytold/wordcraft (the `upstream` remote).
[`CLAUDE.md`](CLAUDE.md) and [`AGENTS.md`](AGENTS.md) are upstream's files and stay unchanged here;
everything in them still applies (never crash, no Microsoft assets, clean room, quality gates).
This file says where the fork differs. Read it and [`ROADMAP-aniefork.md`](ROADMAP-aniefork.md) at
the start of a session. The sister fork arcanii/pdfcraft follows the same rules.

**What kind of fork this is:** one kept only to propose changes back to upstream (the owner,
2026-10-14: "This fork was for developing features for upstream"). Upstream's brand licence
([`docs/brand/LICENSE-brand.txt`](docs/brand/LICENSE-brand.txt), "Forks and modified versions")
lets such a fork keep the ArtCraft marks unchanged, so `README.md` and the files in `docs/brand/`
stay upstream's here. That holds only while the fork gives out nothing of its own: no releases or
builds, and no feature kept for the fork's own sake. Before either of those, the marks must be
removed and the fork must stop presenting itself as an ArtCraft product, as the sister fork of
gridcraft (arcanii/DevCraft) did on 2026-10-14.

At present the fork carries no code of its own: its one feature, the caret's glide, went upstream
(storytold/wordcraft#570) and the fork's copy was removed. The rules below are for the next
fork-only change; `git show 2648505:crates/ui-egui/src/aniefork/mod.rs` and its neighbours show
how the module, its hooks, its settings and its string table were laid out.

- **Log fork work in `ROADMAP-aniefork.md`, not in upstream's documents.** `ROADMAP.md`, `docs/`,
  `CLAUDE.md` and `AGENTS.md` stay identical to upstream so that merging upstream doesn't conflict.
  Where `CLAUDE.md` says to update the roadmap and parity documents when work lands, add a line to
  the log in `ROADMAP-aniefork.md` instead, and keep its "What the fork carries" table true.
- **Fork-private files carry `aniefork` in their path**, so upstream can never add the same one:
  - a document sits beside the upstream document it stands in for, with a `-aniefork` suffix:
    `ROADMAP.md` → `ROADMAP-aniefork.md`, `AGENTS.md` → `AGENTS-aniefork.md`,
    `docs/ui-parity.md` → `docs/ui-parity-aniefork.md`;
  - code goes in an `aniefork` module of the crate it belongs to
    (`crates/ui-egui/src/aniefork/`), with its tests.
- **Upstream files get hooks only:** a `mod` line, or a short call into the fork's module, with
  `(aniefork)` in the comment above it. Keep the logic on the fork's side of the call. Every hook is
  listed in `ROADMAP-aniefork.md`, so a merge conflict is settled by taking upstream's code and
  putting the hook back.
- **State the UI needs between frames** lives in the fork's module (egui's temporary data, keyed by
  an `aniefork.*` id) rather than in new fields on upstream's structs. **Settings that are saved**
  go in an `aniefork::Prefs` struct, the one field the fork adds to `UiState`.
- **Interface text the fork adds** is translated in the fork's own table
  (`crates/ui-egui/src/aniefork/strings.rs`, looked up with `aniefork::strings::t`), into every
  interface language, never in upstream's catalogs (`i18n/*.tsv` must all hold the same entries,
  and upstream changes them constantly). Translate from the meaning, with the words upstream's
  catalogs already use; never from Microsoft's or another product's localisation.
- **A branch meant as an upstream pull request** follows upstream's rules, its progress documents
  included. Cut it from `upstream/main`, not from the fork's `main`.
- **Syncing with upstream:** `ROADMAP-aniefork.md` lists the steps and what can still conflict.
