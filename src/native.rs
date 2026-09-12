//! Native activation delivery. Work runs away from the event-loop thread;
//! callbacks never interpret an activation as a shell command or arbitrary URL.
use crate::{
    activation::OpenRequest,
    error::{AppError, AppResult, ErrorCode},
    runtime, storage, windowing,
};
use std::{
    collections::{HashSet, VecDeque},
    sync::Mutex,
};
use tauri::{Emitter, Manager};

pub const FAILURE_EVENT: &str = "local-store://activation-failure";
#[derive(Default)]
pub struct NativeState {
    active: Mutex<HashSet<String>>,
    failures: Mutex<VecDeque<AppError>>,
}

pub fn focus_launcher(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("launcher") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn report_failure(app: &tauri::AppHandle, error: AppError) {
    let state = app.state::<NativeState>();
    let mut failures = state.failures.lock().unwrap_or_else(|e| e.into_inner());
    if failures.len() == 32 {
        failures.pop_front();
    }
    failures.push_back(error);
    drop(failures);
    focus_launcher(app);
    // The event is a wakeup only. The guarded command drains the queue, so
    // failures arriving before the webview listener exists are not lost.
    let _ = app.emit_to(
        tauri::EventTarget::webview_window("launcher"),
        FAILURE_EVENT,
        (),
    );
}

#[tauri::command]
pub fn take_activation_errors(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
) -> AppResult<Vec<AppError>> {
    crate::commands::require_launcher(&window)?;
    Ok(app
        .state::<NativeState>()
        .failures
        .lock()
        .map_err(AppError::internal)?
        .drain(..)
        .collect())
}

struct ActiveRequest {
    app: tauri::AppHandle,
    id: String,
}
impl Drop for ActiveRequest {
    fn drop(&mut self) {
        self.app
            .state::<NativeState>()
            .active
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&self.id);
    }
}

pub fn dispatch(app: &tauri::AppHandle, request: OpenRequest) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let result = (|| {
            let registry = storage::load_or_migrate_registry().map_err(AppError::from)?;
            let selected = request.resolve(&registry.apps)?;
            if !app
                .state::<NativeState>()
                .active
                .lock()
                .map_err(AppError::internal)?
                .insert(selected.id.clone())
            {
                return Ok(()); // Coalesce duplicate OS delivery while opening.
            }
            let _active = ActiveRequest {
                app: app.clone(),
                id: selected.id.clone(),
            };
            if app
                .get_webview_window(&format!("app-{}", selected.id))
                .is_none()
                && selected.is_managed()
            {
                runtime::start(selected)?;
            }
            windowing::build_window(
                &app,
                &selected.id,
                &selected.display_name,
                &selected.launch_url,
                selected.icon_path.as_ref().and_then(|p| p.to_str()),
            )
            .map_err(|e| AppError::new(ErrorCode::WindowOpenFailed, e))
        })();
        if let Err(error) = result {
            report_failure(&app, error);
        }
    });
}

/// The deep-link feature forwards protocol arguments before this callback.
/// Handling those arguments again here would open the same app twice.
pub fn secondary_request(args: &[String]) -> AppResult<Option<OpenRequest>> {
    let forwarded = args.get(1..).unwrap_or_default();
    // Upstream 2.4.4 serializes Windows argv with an unescaped `|` delimiter.
    // Native opens have already passed exact-arity validation before plugin
    // startup, so restore delimiters inside their one target argument here.
    #[cfg(windows)]
    if forwarded.len() > 2 && forwarded[0] == "open" {
        return crate::activation::request_from_args(&["open".into(), forwarded[1..].join("|")]);
    }
    #[cfg(windows)]
    if forwarded.len() > 1
        && [crate::brand::URL_SCHEME, crate::brand::LEGACY_URL_SCHEME]
            .iter()
            .any(|scheme| forwarded[0].starts_with(&format!("{scheme}:")))
    {
        // In this case the plugin could not recognize a single URI argument.
        return crate::activation::parse_deep_link(&forwarded.join("|")).map(Some);
    }
    if forwarded.len() == 1 && forwarded[0].contains(":") {
        return Ok(None);
    }
    crate::activation::request_from_args(forwarded)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn secondary_cli_is_routed_but_protocol_is_left_to_the_plugin() {
        let args = |v: &[&str]| v.iter().map(|s| (*s).into()).collect::<Vec<String>>();
        assert!(
            secondary_request(&args(&["binary", "localstore://open/memos"]))
                .unwrap()
                .is_none()
        );
        assert!(secondary_request(&args(&["binary", "open", "memos"]))
            .unwrap()
            .is_some());
        assert!(secondary_request(&args(&["binary"])).unwrap().is_none());
    }

    #[cfg(windows)]
    #[test]
    fn windows_forwarding_preserves_pipe_in_display_name() {
        let args = |v: &[&str]| v.iter().map(|s| (*s).into()).collect::<Vec<String>>();
        let legacy = format!("{}://open/Notes%20", crate::brand::LEGACY_URL_SCHEME);
        for parts in [
            vec!["binary", "open", "Notes ", " work"],
            vec!["binary", &legacy, "%20work"],
        ] {
            assert_eq!(
                secondary_request(&args(&parts)).unwrap().unwrap().target,
                "Notes | work"
            );
        }
    }
}
