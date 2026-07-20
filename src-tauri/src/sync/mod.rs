//! Two-way calendar / reminders sync.
//!
//! A `SyncProvider` trait + factory (mirroring `ai/`) abstracts the four
//! backends — Apple Calendar, Apple Reminders, Google Calendar, Google Tasks —
//! behind one [`engine`] that treats `acorn.db` as the source of truth and
//! reconciles providers against it.

pub mod engine;
pub mod error;
pub mod keychain;
pub mod mapping;
pub mod provider;
pub mod providers;
pub mod resolve;
pub mod scheduler;
pub mod types;
