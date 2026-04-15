// crates/wayback-detect/src/lib.rs

//! wayback-detect
//!
//! YAML-driven engine detection for the Wayback / Internet-of-Games stack.
//!
//! This crate loads signature definitions from config/wayback/signatures.yaml
//! and exposes a simple API:
//!
//!   - Detector::from_path("config/wayback/signatures.yaml")?
//!   - detector.detect("udp", &packet_bytes)
//!
//! The goal is to answer: "what engine/game-family does this payload look like?"
//! so that wayback-proxy can route it to the right handlers.
//!
//! The YAML format is defined in docs and exemplified in
//! config/wayback/signatures.yaml.

mod signatures;

pub use crate::signatures::{Detector, DetectError, DetectResult, EngineId, Signature};
