//! Pepometer's core: the Garmin client, local storage, sync, and the rules
//! that turn Garmin's numbers into the screens in `view`.
//!
//! The same crate runs inside the Android app (foreground and the WorkManager
//! job), the developer CLI, and later a self-hosted server for the web app.

pub mod view;
