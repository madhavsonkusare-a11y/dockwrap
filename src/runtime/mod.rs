//! Runtime process code: Docker Compose boot and health polling.

use crate::model::AppDef;
use std::time::{Duration, Instant};

/// Boot the app's compose stack (if any) and wait until its health target responds.
/// Non-fatal by design: callers should still open the window even if this errors.
pub fn boot_and_wait(app: &AppDef) -> Result<(), String> {
    if let Some(compose) = &app.compose {
        // Use -f for an explicit file; if it's a directory, docker falls back to
        // compose.yml/compose.yaml inside it.
        let status = std::process::Command::new("docker")
            .args(["compose", "-f", compose, "up", "-d"])
            .status();
        match status {
            Ok(s) if s.success() => {}
            Ok(s) => return Err(format!("docker compose up exited {}", s)),
            Err(e) => return Err(format!("failed to spawn docker: {}", e)),
        }
    }

    let target = app.health.clone().unwrap_or_else(|| app.url.clone());
    wait_for_health(&target, Duration::from_secs(60))
}

/// Poll `target` (an http(s):// URL or host:port) until it responds or the
/// deadline passes. Uses curl (present on Windows/macOS/Linux) for a portable,
/// dependency-free check.
pub fn wait_for_health(target: &str, timeout: Duration) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    loop {
        if is_up(target) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!("health timeout after {:?} for {}", timeout, target));
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

fn is_up(target: &str) -> bool {
    std::process::Command::new("curl")
        .args(["-sf", "--max-time", "2", "--", target])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Validated OS URL opening through the maintained platform integration crate.
pub fn launch_browser(url: &str) {
    match crate::windowing::validated_external_url(url) {
        Ok(url) => {
            if let Err(error) = open::that_detached(url) {
                eprintln!("Could not open browser: {error}");
            }
        }
        Err(error) => eprintln!("Could not open browser: {error}"),
    }
}
