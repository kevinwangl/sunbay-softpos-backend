// Test library root
// This file allows tests to access the main crate's modules

#[cfg(test)]
mod unit;

#[cfg(test)]
mod integration;

// Re-export main crate modules for testing
pub use sunbay_softpos_backend::*;
