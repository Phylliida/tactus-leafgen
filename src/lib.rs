//! tactus-leafgen — procedural leaf generator.
//!
//! Phase 1 (current): pure-Rust prototype, `f64` arithmetic, no proofs.
//! The goal of this phase is to get the *algorithms* right and visually
//! botanically plausible. Verification (tactus / Lean backend) comes later;
//! when it does, the numeric type (`Scalar`) becomes the seam we generalize:
//! `f64` for the fast path, exact `Rational` for the proofs, and eventually
//! `f64` + a flocq rounding model for verified-fast geometry.

pub mod vec2;
pub mod rng;
pub mod blade;
pub mod major;
pub mod palmate;
pub mod compound;
pub mod venation;
pub mod svg;
pub mod raster;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

/// The scalar type for all geometry. Today this is `f64`; it is the single
/// point we will generalize when verification begins (see module docs).
pub type Scalar = f64;
