//! Unit test modules for individual components
//!
//! This module contains unit tests for core types, utilities, and component-specific logic.
//! All code here is test infrastructure -- imports and helpers are only used under `#[cfg(test)]`.

#![allow(unused_imports)]

pub mod core_types;
pub mod core_utils;
pub mod database;
pub mod rbac;
pub mod repository;
pub mod search;
pub mod session;
