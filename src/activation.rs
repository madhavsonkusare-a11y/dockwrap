//! Native activation routing shared by startup and future existing-instance delivery.
//! Parsing never touches the registry, Docker, protocol handlers or windows.
use crate::{
    brand::{LEGACY_URL_SCHEME, URL_SCHEME},
    error::{AppError, AppResult, ErrorCode},
    model::InstalledApp,
    windowing,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenRequest {
    pub target: String,
    pub allow_display_name: bool,
}
impl OpenRequest {
    pub fn resolve<'a>(&self, apps: &'a [InstalledApp]) -> AppResult<&'a InstalledApp> {
        // Stable IDs always win over another app's coincidentally matching name.
        let app = apps
            .iter()
            .find(|app| app.id == self.target)
            .or_else(|| {
                self.allow_display_name
                    .then(|| {
                        apps.iter()
                            .find(|app| app.display_name.eq_ignore_ascii_case(&self.target))
                    })
                    .flatten()
            })
            .ok_or_else(|| {
                AppError::new(
                    ErrorCode::NotFound,
                    format!("No app with ID or name {:?} found.", self.target),
                )
            })?;
        windowing::validated_external_url(&app.launch_url).map_err(AppError::invalid)?;
        Ok(app)
    }
}

fn valid_target(target: &str) -> bool {
    !target.trim().is_empty()
        && target.len() <= 2048
        && !target
            .chars()
            .any(|c| c.is_control() || c == '/' || c == '\\')
        && target != "."
        && target != ".."
}

pub fn parse_deep_link(input: &str) -> AppResult<OpenRequest> {
    let invalid = || AppError::invalid("Use a localstore://open/<app-id> link with one app ID.");
    if input.len() > 4096 || input.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(invalid());
    }
    let url = tauri::Url::parse(input).map_err(|_| invalid())?;
    if ![URL_SCHEME, LEGACY_URL_SCHEME].contains(&url.scheme())
        || url.host_str() != Some("open")
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid());
    }
    // Examine the raw path too: URL parsing normalizes dot segments and must
    // not turn `a/../b` into an accepted request for a different app.
    let (_, authority_path) = input.split_once("://").ok_or_else(invalid)?;
    let (_, raw_path) = authority_path.split_once('/').ok_or_else(invalid)?;
    if raw_path.contains('/') {
        return Err(invalid());
    }
    let target = windowing::strict_percent_decode_path_segment(raw_path).ok_or_else(invalid)?;
    if !valid_target(&target) {
        return Err(invalid());
    }
    Ok(OpenRequest {
        target,
        allow_display_name: url.scheme() == LEGACY_URL_SCHEME,
    })
}

/// None means ordinary launcher startup or CLI dispatch. A recognized malformed
/// protocol request is an error, never a request to execute CLI arguments.
pub fn request_from_args(args: &[String]) -> AppResult<Option<OpenRequest>> {
    if let Some(first) = args.first() {
        let scheme = first.split_once(':').map(|(scheme, _)| scheme);
        if scheme.is_some_and(|scheme| {
            [URL_SCHEME, LEGACY_URL_SCHEME]
                .iter()
                .any(|s| scheme.eq_ignore_ascii_case(s))
        }) {
            if args.len() != 1 {
                return Err(AppError::invalid(
                    "A protocol activation accepts exactly one URL.",
                ));
            }
            return parse_deep_link(first).map(Some);
        }
    }
    match args {
        [command, target] if command == "open" && !target.starts_with('-') => {
            if target.trim().is_empty()
                || target.len() > 2048
                || target.chars().any(char::is_control)
            {
                return Err(AppError::invalid(
                    "Use a non-empty app ID or name without control characters.",
                ));
            }
            Ok(Some(OpenRequest {
                target: target.clone(),
                allow_display_name: true,
            }))
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::RuntimeSpec;
    fn app(id: &str, name: &str) -> InstalledApp {
        InstalledApp {
            id: id.into(),
            display_name: name.into(),
            launch_url: "http://localhost:5230".into(),
            runtime: RuntimeSpec::External,
            catalog_id: None,
            icon_path: None,
            created_at_unix: 1,
            updated_at_unix: 1,
        }
    }
    #[test]
    fn primary_ids_and_legacy_names_have_distinct_resolution_rules() {
        let apps = vec![app("other", "memos"), app("memos", "My notes")];
        assert_eq!(
            parse_deep_link("localstore://open/memos")
                .unwrap()
                .resolve(&apps)
                .unwrap()
                .id,
            "memos"
        );
        assert_eq!(
            parse_deep_link(&format!("{LEGACY_URL_SCHEME}://open/My%20notes"))
                .unwrap()
                .resolve(&apps)
                .unwrap()
                .id,
            "memos"
        );
        assert_eq!(
            parse_deep_link("localstore://open/My%20notes")
                .unwrap()
                .resolve(&apps)
                .unwrap_err()
                .code,
            ErrorCode::NotFound
        );
        assert_eq!(
            parse_deep_link("localstore://open/Notes%2BMore")
                .unwrap()
                .target,
            "Notes+More"
        );
    }
    #[test]
    fn malformed_activations_cannot_be_normalized_into_valid_targets() {
        for input in [
            "https://open/memos",
            "localstore://wrong/memos",
            "localstore://open/",
            "localstore://open/a/../memos",
            "localstore://open/%2e%2e/memos",
            "localstore://open/memos/extra",
            "localstore://open/a%2fb",
            "localstore://open/a%5cb",
            "localstore://open/%00",
            "localstore://open/%20",
            "localstore://open/%FF",
            "localstore://open/%ZZ",
            "localstore://open/%",
            "localstore://open/memos?x=1",
            "localstore://open/memos#x",
            "localstore://user@open/memos",
            "localstore://open:123/memos",
        ] {
            assert_eq!(
                parse_deep_link(input).unwrap_err().code,
                ErrorCode::InvalidInput,
                "{input}"
            );
        }
    }
    #[test]
    fn only_exact_native_open_requests_bypass_cli() {
        let args = |a: &[&str]| a.iter().map(|v| (*v).into()).collect::<Vec<String>>();
        assert!(request_from_args(&args(&["open", "My notes"]))
            .unwrap()
            .is_some());
        assert!(request_from_args(&args(&["open", "Notes / Work"]))
            .unwrap()
            .is_some());
        for values in [
            vec!["open", "memos", "--browser"],
            vec!["open", "--help"],
            vec!["doctor", "--json"],
            vec![],
        ] {
            assert_eq!(request_from_args(&args(&values)).unwrap(), None);
        }
        assert!(request_from_args(&args(&["localstore://open/memos", "extra"])).is_err());
        assert!(request_from_args(&args(&["open", " "])).is_err());
    }
    #[test]
    fn invalid_saved_urls_fail_before_runtime_start() {
        let mut saved = app("memos", "Memos");
        saved.launch_url = "file:///private".into();
        assert_eq!(
            parse_deep_link("localstore://open/memos")
                .unwrap()
                .resolve(&[saved])
                .unwrap_err()
                .code,
            ErrorCode::InvalidInput
        );
    }
}
