//! wired patch: Skia-style text rendering.
//!
//! Reproduces how Chromium on Windows rasterizes text into a transparent canvas (the path used by
//! xterm.js-based terminals such as Tabby): DirectWrite ClearType 3x1 coverage averaged to one
//! channel, then Skia's mask gamma. Ported from Skia, see `LICENSE-SKIA`:
//! - `https://github.com/google/skia/blob/815b987b2ca82b80993ebc08a309a5275522ab0e/src/ports/SkScalerContext_win_dw.cpp` (`RGBToA8`, grid fit selection)
//! - `https://github.com/google/skia/blob/815b987b2ca82b80993ebc08a309a5275522ab0e/src/core/SkMaskGamma.cpp` (`SkTMaskGamma_build_correcting_lut`)
//! - `https://github.com/google/skia/blob/815b987b2ca82b80993ebc08a309a5275522ab0e/src/core/SkMaskGamma.h` (`CanonicalColor`, `sk_t_scale255`)
//! Default contrast and gamma follow Chromium, which takes them from DirectWrite's rendering
//! params: `https://github.com/chromium/chromium/blob/8999a25190b5252cfb419516a28b9e38d1c5f4ce/ui/gfx/font_util_win.cc`.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

const UNSET: u32 = u32::MAX;
static SKIA_TEXT: AtomicBool = AtomicBool::new(false);
static CONTRAST: AtomicU32 = AtomicU32::new(UNSET);
static GAMMA: AtomicU32 = AtomicU32::new(UNSET);
static SYSTEM_CONTRAST: AtomicU32 = AtomicU32::new(0);
static SYSTEM_GAMMA: AtomicU32 = AtomicU32::new(0);

/// Turns Skia-style text on or off. `contrast` and `gamma` override the system DirectWrite values
/// (`None` = use them, like Chromium). Call before the first window is created.
pub fn set_skia_text(enabled: bool, contrast: Option<f32>, gamma: Option<f32>) {
    SKIA_TEXT.store(enabled, Ordering::Relaxed);
    CONTRAST.store(contrast.map_or(UNSET, f32::to_bits), Ordering::Relaxed);
    GAMMA.store(gamma.map_or(UNSET, f32::to_bits), Ordering::Relaxed);
}

pub(crate) fn skia_text() -> bool {
    SKIA_TEXT.load(Ordering::Relaxed)
}

pub(crate) fn record_system_text_params(gamma: f32, enhanced_contrast: f32) {
    SYSTEM_GAMMA.store(gamma.to_bits(), Ordering::Relaxed);
    SYSTEM_CONTRAST.store(enhanced_contrast.to_bits(), Ordering::Relaxed);
}

fn load(cell: &AtomicU32, fallback: &AtomicU32) -> f32 {
    match cell.load(Ordering::Relaxed) {
        UNSET => f32::from_bits(fallback.load(Ordering::Relaxed)),
        bits => f32::from_bits(bits),
    }
}

/// `[enabled, contrast bits, gamma bits]` for the shaders' `global_pad`. Clamped to the ranges of
/// `SkSurfaceProps` (contrast [0, 1], gamma [0, 4)); gamma below 1 is not meaningful here.
pub(crate) fn skia_shader_params() -> [u32; 3] {
    let contrast = load(&CONTRAST, &SYSTEM_CONTRAST).clamp(0.0, 1.0);
    let gamma = load(&GAMMA, &SYSTEM_GAMMA).clamp(1.0, 3.99);
    [u32::from(skia_text()), contrast.to_bits(), gamma.to_bits()]
}
