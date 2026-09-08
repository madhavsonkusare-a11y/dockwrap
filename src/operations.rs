//! Launcher-only operation notifications. IPC remains the source of truth;
//! a failed event delivery must never make a successful mutation look failed.
use crate::error::{AppError, AppResult, ErrorCode};
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::Emitter;

static NEXT: AtomicU64 = AtomicU64::new(1);

pub const EVENT_NAME: &str = "local-store://operation";
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    Install,
    Start,
    Stop,
    Uninstall,
    Open,
}
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationState {
    Started,
    Progress,
    Succeeded,
    Failed,
    Cancelled,
}
#[derive(Debug, Clone, Serialize)]
pub struct OperationEvent {
    pub operation_id: String,
    pub app_id: String,
    pub kind: OperationKind,
    pub state: OperationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AppError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<crate::runtime::InstallStage>,
    /// Names the runtime operation a cancel request must target. Absent when
    /// the operation cannot be cancelled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_id: Option<u64>,
}

pub async fn track<T>(
    window: &tauri::WebviewWindow,
    app_id: &str,
    kind: OperationKind,
    work: impl std::future::Future<Output = AppResult<T>>,
) -> AppResult<T> {
    // Defense in depth: never send payloads to remote application windows.
    crate::commands::require_launcher(window)?;
    track_with(
        app_id,
        kind,
        |event| {
            let _ = window.emit_to(
                tauri::EventTarget::webview_window("launcher"),
                EVENT_NAME,
                event,
            );
        },
        work,
    )
    .await
}

async fn track_with<T>(
    app_id: &str,
    kind: OperationKind,
    emit: impl Fn(&OperationEvent),
    work: impl std::future::Future<Output = AppResult<T>>,
) -> AppResult<T> {
    let mut event = OperationEvent {
        operation_id: format!(
            "{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ),
        app_id: app_id.into(),
        kind,
        state: OperationState::Started,
        error: None,
        stage: None,
        // Only installs are cancellable today; the short lifecycle commands
        // finish faster than a cancel could reach them.
        cancel_id: None,
    };
    emit(&event);
    let result = work.await;
    event.state = match &result {
        Ok(_) => OperationState::Succeeded,
        Err(error) if error.code == ErrorCode::Cancelled => OperationState::Cancelled,
        Err(_) => OperationState::Failed,
    };
    event.error = result.as_ref().err().cloned();
    emit(&event);
    result
}

/// Stage callbacks run on the blocking install worker; events stay launcher-only.
pub async fn track_install<T, F>(
    window: &tauri::WebviewWindow,
    app_id: &str,
    cancel_id: Option<u64>,
    work: impl FnOnce(std::sync::Arc<dyn Fn(crate::runtime::InstallStage) + Send + Sync>) -> F,
) -> AppResult<T>
where
    F: std::future::Future<Output = AppResult<T>>,
{
    crate::commands::require_launcher(window)?;
    let mut event = OperationEvent {
        operation_id: format!(
            "{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ),
        app_id: app_id.into(),
        kind: OperationKind::Install,
        state: OperationState::Started,
        error: None,
        stage: None,
        cancel_id,
    };
    let emit_window = window.clone();
    let emit = move |event: &OperationEvent| {
        let _ = emit_window.emit_to(
            tauri::EventTarget::webview_window("launcher"),
            EVENT_NAME,
            event,
        );
    };
    emit(&event);
    let progress_event = event.clone();
    let progress_emit = emit.clone();
    let result = work(std::sync::Arc::new(move |stage| {
        let mut next = progress_event.clone();
        next.state = OperationState::Progress;
        next.stage = Some(stage);
        progress_emit(&next);
    }))
    .await;
    event.state = match &result {
        Ok(_) => OperationState::Succeeded,
        Err(error) if error.code == ErrorCode::Cancelled => OperationState::Cancelled,
        Err(_) => OperationState::Failed,
    };
    event.error = result.as_ref().err().cloned();
    emit(&event);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    #[test]
    fn events_correlate_and_preserve_terminal_results() {
        let mut ids = Vec::new();
        for result in [
            Ok(()),
            Err(AppError::new(ErrorCode::Cancelled, "cancelled")),
            Err(AppError::new(
                ErrorCode::TimedOut,
                "https://admin:secret@example.com timed out",
            )),
        ] {
            let events = Mutex::new(Vec::new());
            let returned = tauri::async_runtime::block_on(track_with(
                "memos",
                OperationKind::Start,
                |event| events.lock().unwrap().push(event.clone()),
                async { result.clone() },
            ));
            assert_eq!(returned, result);
            let events = events.into_inner().unwrap();
            assert_eq!(events.len(), 2);
            assert_eq!(events[0].state, OperationState::Started);
            assert_eq!(events[0].operation_id, events[1].operation_id);
            assert_eq!(events[1].app_id, "memos");
            assert_eq!(
                events[1].state,
                match result {
                    Ok(_) => OperationState::Succeeded,
                    Err(ref e) if e.code == ErrorCode::Cancelled => OperationState::Cancelled,
                    Err(_) => OperationState::Failed,
                }
            );
            assert_eq!(events[1].error, result.err());
            assert!(!serde_json::to_string(&events).unwrap().contains("secret"));
            ids.push(events[0].operation_id.clone());
        }
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn lifecycle_events_carry_no_cancel_id() {
        let events = Mutex::new(Vec::new());
        let _ = tauri::async_runtime::block_on(track_with(
            "memos",
            OperationKind::Start,
            |event| events.lock().unwrap().push(event.clone()),
            async { Ok::<(), AppError>(()) },
        ));
        let events = events.into_inner().unwrap();
        assert!(events.iter().all(|event| event.cancel_id.is_none()));
        // A consumer must not be offered a control the backend cannot honour.
        let encoded = serde_json::to_string(&events[0]).unwrap();
        assert!(!encoded.contains("cancel_id"), "{encoded}");
    }

    #[test]
    fn an_install_event_names_the_operation_a_cancel_must_target() {
        let event = OperationEvent {
            operation_id: "install-1".into(),
            app_id: "memos".into(),
            kind: OperationKind::Install,
            state: OperationState::Started,
            error: None,
            stage: None,
            cancel_id: Some(42),
        };
        let encoded = serde_json::to_value(&event).unwrap();
        assert_eq!(encoded["cancel_id"], 42);
        assert_eq!(encoded["kind"], "install");
        assert_eq!(encoded["state"], "started");
    }
}
