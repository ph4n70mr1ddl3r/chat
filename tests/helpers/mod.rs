//! Test Helpers - Utilities for better test patterns
//!
//! This module provides reusable test utilities that improve test quality:
//! - polling: Deterministic wait patterns (replace hard sleeps)
//! - factories: Data factory functions (replace hardcoded test data)
//! - fixtures: Shared test fixtures (reduce setup duplication)

pub mod polling;
pub mod factories;
