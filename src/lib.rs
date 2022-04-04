//! # taskmr
//!
//! taskmr is a Task ManageR.
//! taskmr supplys the simple and efficient way to manage tasks.
//!
//! Bellow modules are layers based on Onion Architecture.

/// domain is a layer which represent business rules.
pub mod domain;
/// infra is a infrastructure layer.
pub mod infra;
/// presentation is a layer which is transrate from/to any UI.
pub mod presentation;
/// usecase is a layer which represent use case.
pub mod usecase;
