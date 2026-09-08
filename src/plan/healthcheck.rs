//! Container health checks, distinct from the launcher's HTTP readiness probe.
//! Imported commands run only inside the container, never during import.
use super::{scalar, PlanArgs};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthTest {
    Disabled,
    Exec(Vec<String>),
    Shell(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn compose_modes_and_image_inheritance_remain_distinct() {
        for (input, expected) in [
            (
                json!({"test":["CMD", "curl", "-f", "http://localhost"]}),
                HealthTest::Exec(vec!["curl".into(), "-f".into(), "http://localhost".into()]),
            ),
            (
                json!({"test":"curl -f http://localhost || exit 1"}),
                HealthTest::Shell("curl -f http://localhost || exit 1".into()),
            ),
            (
                json!({"test":["CMD-SHELL", "exit 0"]}),
                HealthTest::Shell("exit 0".into()),
            ),
            (json!({"test":["NONE"]}), HealthTest::Disabled),
            (json!({"disable":true}), HealthTest::Disabled),
        ] {
            assert_eq!(
                PlanHealthcheck::from_compose(&input).unwrap().test,
                Some(expected)
            );
        }
        let inherited = PlanHealthcheck::from_compose(&json!({"interval":"1m30s", "timeout":"0.5s", "start_period":"0s", "start_interval":"2s", "retries":3})).unwrap();
        assert_eq!(inherited.test, None);
        assert_eq!(inherited.interval.as_deref(), Some("1m30s"));
    }

    #[test]
    fn malformed_or_unresolved_checks_fail_without_echoing_commands() {
        for input in [
            json!({"unknown":true}),
            json!({"test":["CMD"]}),
            json!({"test":["CMD-SHELL","exit", "0"]}),
            json!({"test":["NONE", "private"]}),
            json!({"test":"echo $PRIVATE"}),
            json!({"test":"echo $$PRIVATE"}),
            json!({"test":"echo\nprivate"}),
            json!({"test":["CMD", 3]}),
            json!({"interval":"-1s"}),
            json!({"timeout":"forever"}),
            json!({"retries":0}),
            json!({"retries":1001}),
            json!({"retries":-1}),
            json!({"disable":true,"test":"exit 0"}),
        ] {
            let error = PlanHealthcheck::from_compose(&input).unwrap_err();
            assert!(!error.contains("PRIVATE") && !error.contains("private"));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlanHealthcheck {
    pub test: Option<HealthTest>,
    pub interval: Option<String>,
    pub timeout: Option<String>,
    pub start_period: Option<String>,
    pub start_interval: Option<String>,
    pub retries: Option<u32>,
}

impl PlanHealthcheck {
    /// Parse Compose's healthcheck object without dropping unknown options.
    pub fn from_compose(value: &serde_json::Value) -> Result<Self, String> {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            test: Option<serde_json::Value>,
            disable: Option<bool>,
            interval: Option<String>,
            timeout: Option<String>,
            start_period: Option<String>,
            start_interval: Option<String>,
            retries: Option<u32>,
        }
        let raw: Raw = serde_json::from_value(value.clone())
            .map_err(|_| "Unsupported healthcheck property or value type")?;
        let test = match raw.test {
            None => None,
            Some(serde_json::Value::String(command)) => Some(HealthTest::Shell(command)),
            Some(serde_json::Value::Array(parts)) => {
                let parts = parts
                    .iter()
                    .map(|p| p.as_str().map(str::to_owned))
                    .collect::<Option<Vec<_>>>()
                    .ok_or("Healthcheck test must contain text")?;
                match parts.first().map(String::as_str) {
                    Some("NONE") if parts.len() == 1 => Some(HealthTest::Disabled),
                    Some("CMD") if parts.len() > 1 => Some(HealthTest::Exec(parts[1..].to_vec())),
                    Some("CMD-SHELL") if parts.len() == 2 => {
                        Some(HealthTest::Shell(parts[1].clone()))
                    }
                    _ => return Err("Unsupported healthcheck test mode or arity".into()),
                }
            }
            Some(_) => return Err("Unsupported healthcheck test type".into()),
        };
        if raw.disable == Some(true) && test.is_some() {
            return Err("Conflicting healthcheck disable and test".into());
        }
        let check = Self {
            test: if raw.disable == Some(true) {
                Some(HealthTest::Disabled)
            } else {
                test
            },
            interval: raw.interval,
            timeout: raw.timeout,
            start_period: raw.start_period,
            start_interval: raw.start_interval,
            retries: raw.retries,
        };
        check.validate()?;
        Ok(check)
    }

    pub fn validate(&self) -> Result<(), String> {
        let parts = match &self.test {
            Some(HealthTest::Exec(parts)) => {
                PlanArgs::Exec(parts.clone()).validate("healthcheck")?;
                parts.iter().map(String::as_str).collect::<Vec<_>>()
            }
            Some(HealthTest::Shell(command)) => {
                PlanArgs::Shell(command.clone()).validate("healthcheck")?;
                vec![command.as_str()]
            }
            _ => vec![],
        };
        // Setup currently resolves environment only. Refuse expressions here
        // rather than freezing a secret or confusing Compose/container expansion.
        if parts.iter().any(|p| p.contains('$')) {
            return Err("Healthcheck variable expansion is not modelled".into());
        }
        let duration =
            regex::Regex::new(r"^(?:[0-9]{1,6}(?:\.[0-9]{1,6})?(?:ns|us|ms|s|m|h)){1,8}$").unwrap();
        for value in [
            &self.interval,
            &self.timeout,
            &self.start_period,
            &self.start_interval,
        ]
        .into_iter()
        .flatten()
        {
            if value.len() > 128 || !duration.is_match(value) {
                return Err("Unsupported healthcheck duration".into());
            }
        }
        if self.retries.is_some_and(|value| value == 0 || value > 1000) {
            return Err("Healthcheck retries must be between 1 and 1000".into());
        }
        Ok(())
    }

    pub(super) fn render(&self, out: &mut String) {
        out.push_str("    healthcheck:\n");
        match &self.test {
            Some(HealthTest::Disabled) => out.push_str("      disable: true\n"),
            Some(HealthTest::Shell(command)) => out.push_str(&format!(
                "      test: [\"CMD-SHELL\", {}]\n",
                scalar(command)
            )),
            Some(HealthTest::Exec(parts)) => {
                out.push_str("      test:\n        - \"CMD\"\n");
                for part in parts {
                    out.push_str(&format!("        - {}\n", scalar(part)));
                }
            }
            None => {}
        }
        for (key, value) in [
            ("interval", &self.interval),
            ("timeout", &self.timeout),
            ("start_period", &self.start_period),
            ("start_interval", &self.start_interval),
        ] {
            if let Some(value) = value {
                out.push_str(&format!("      {key}: {}\n", scalar(value)));
            }
        }
        if let Some(retries) = self.retries {
            out.push_str(&format!("      retries: {retries}\n"));
        }
        // An empty mapping inherits the image health check.
        if self == &Self::default() {
            out.push_str("      disable: false\n");
        }
    }
}
