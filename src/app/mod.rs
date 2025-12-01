//! Application module structure for SlircApp
//!
//! This module organizes the main application into focused submodules:
//! - `core`: SlircApp struct and initialization
//! - `events`: Event processing from backend
//! - `update`: Main update loop and global shortcuts
//! - `dialogs`: Dialog rendering orchestration
//! - `ui::panels`: Menu bar, toolbar, and central panel rendering
//! - `ui::input`: Message input panel with history and completion
//! - `ui::menus`: Context menus and floating windows

pub mod core;
pub mod dialogs;
pub mod events;
pub mod update;
pub mod ui;

// Re-export SlircApp for public API
pub use core::SlircApp;
