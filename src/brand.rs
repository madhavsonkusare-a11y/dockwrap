//! Product identity shared by the desktop app, CLI, and platform integrations.

pub const PRODUCT_NAME: &str = "Local Store";
pub const CLI_NAME: &str = "local-store";
pub const CONFIG_SLUG: &str = "local-store";
pub const URL_SCHEME: &str = "localstore";

/// One-release compatibility with the pre-rename configuration directory.
pub const LEGACY_CONFIG_SLUG: &str = "dockwrap";
/// One-release compatibility with pre-rename deep links.
pub const LEGACY_URL_SCHEME: &str = "dockwrap";
