# gpui-pre-wired

[gpui](https://github.com/zed-industries/zed) as republished on crates.io, with patches for the
[wired](https://github.com/twelvehouse/wired-term) terminal emulator:

- `gpui-pre/`: `gpui-pre` 0.3.12, the framework.
- `gpui-pre-windows/`: `gpui-pre-windows` 0.3.12, its Windows platform crate.

Both are snapshots of zed@d2a4fb6 renamed for crates.io, so there is no git history to fork. The
first commit is both crates exactly as published; later commits are the patches. wired uses them
through `[patch.crates-io]`. To update, import the new published versions the same way and
re-apply the patches.

## Patches

All patched code is marked with `wired patch` comments.

### gpui-pre

- **Hover styles follow layout changes** (`src/elements/div.rs`): an element's hover state (and
  group hover state) was only updated by mouse-move events, so an element that moved under a still
  pointer, or away from it, kept the wrong `.hover()` style until the mouse moved. It is now also
  compared with each frame's hit test at paint time, the way the `on_hover` listener already is.

### gpui-pre-windows

- **Skia-style text** (`src/wired_text.rs`, `set_skia_text`): renders text the way Chromium on
  Windows draws into a transparent canvas, which is what xterm.js-based terminals show:
  - DirectWrite ClearType 3x1 coverage averaged to one channel (`src/direct_write.rs`),
  - grid fitting enabled,
  - Skia's mask gamma in the glyph shader (`src/shaders.hlsl`), with contrast and gamma taken from
    DirectWrite's rendering params like Chromium does (`src/directx_renderer.rs`),
  - grayscale even on opaque windows.

  Off by default; with it off the crate behaves as upstream.

- **Color emoji with Skia-style text** (`src/direct_write.rs`): color glyph layers are rasterized
  with `DWRITE_TEXTURE_ALIASED_1x1`, so emoji are drawn instead of left blank.

- **IME candidate window keeps clear of the caret** (`src/events.rs`, `src/window.rs`): the caret
  rectangle is passed with `CFS_EXCLUDE`, as Chromium's `IMM32Manager::MoveImeWindow` does, instead
  of a point at the middle of the line (`CFS_CANDIDATEPOS`). When there is no room below, the IME
  opens the list above the text instead of over it.

## Licenses

- gpui-pre and gpui-pre-windows: Apache-2.0, see `LICENSE-APACHE` in each folder.
- Code ported from [Skia](https://skia.org) (`SkScalerContext_win_dw.cpp`, `SkMaskGamma.cpp`,
  `SkMaskGamma.h`): BSD-3-Clause, Copyright Google Inc., see `gpui-pre-windows/LICENSE-SKIA`. The
  ported parts cite the source files and commit in comments.
- Chromium (`ui/gfx/font_util_win.cc`, `ui/gfx/font_render_params_win.cc`,
  `ui/base/ime/win/imm32_manager.cc`) was used as a reference for parameter choices and API usage
  only; no code is copied from it.
