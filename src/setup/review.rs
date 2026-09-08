//! An explicit UI projection, never serialization of a deployment template.
use super::{FieldKind, PlanTemplate};

#[derive(Debug, serde::Serialize)]
pub struct SetupReview {
    pub fields: Vec<SetupFieldReview>,
    pub service_count: usize,
    pub generated_credential_count: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct SetupFieldReview {
    pub key: String,
    pub label: String,
    pub required: bool,
    pub sensitive: bool,
    pub has_default: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    pub control: &'static str,
    /// Rules are enforced by Rust. Raw regex source never crosses into HTML.
    pub server_validated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<i64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
}

impl PlanTemplate {
    /// Review does not generate/read credentials, resolve answers, access Docker,
    /// or approve an imported template for installation. Approval remains external.
    pub fn setup_review(&self) -> Result<SetupReview, String> {
        self.validate()?;
        let fields = self
            .fields
            .iter()
            .map(|field| {
                let mut view = SetupFieldReview {
                    key: field.key.clone(),
                    label: field.label.clone(),
                    required: field.required,
                    sensitive: field.sensitive,
                    has_default: field.default.is_some(),
                    default: if field.sensitive {
                        None
                    } else {
                        field.default.clone()
                    },
                    control: "text",
                    server_validated: true,
                    min: None,
                    max: None,
                    options: vec![],
                };
                // Sensitive choices/rules can themselves contain credentials. Mask
                // input and leave their validation entirely on the backend.
                if field.sensitive {
                    view.control = "password";
                    return view;
                }
                match &field.kind {
                    FieldKind::Boolean => view.control = "boolean",
                    FieldKind::Number { min, max } => {
                        view.control = "number";
                        view.min = Some(*min);
                        view.max = Some(*max);
                    }
                    FieldKind::Choice { options } => {
                        view.control = "choice";
                        view.options = options.clone();
                    }
                    FieldKind::Text { .. } | FieldKind::Pattern { .. } => {}
                }
                view
            })
            .collect();
        Ok(SetupReview {
            fields,
            service_count: self.plan.services.len(),
            generated_credential_count: self.secrets.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::{SecretFormat, SecretSpec, SetupField};

    fn template() -> PlanTemplate {
        let recipe = crate::recipes::recipe("memos").unwrap();
        let mut plan = crate::plan::plan_for_recipe(&recipe).unwrap();
        plan.services[0].environment.extend([
            ("API_KEY".into(), "${API_KEY}".into()),
            ("DB_PASSWORD".into(), "${DB_PASSWORD}".into()),
            ("SITE_NAME".into(), "${SITE_NAME}".into()),
        ]);
        PlanTemplate {
            plan,
            fields: vec![
                SetupField {
                    key: "API_KEY".into(),
                    label: "API key".into(),
                    required: true,
                    sensitive: true,
                    default: Some("private-default".into()),
                    kind: FieldKind::Choice {
                        options: vec!["private-default".into()],
                    },
                },
                SetupField {
                    key: "SITE_NAME".into(),
                    label: "Site name".into(),
                    required: false,
                    sensitive: false,
                    default: Some("My notes".into()),
                    kind: FieldKind::Text {
                        min_len: 0,
                        max_len: 80,
                    },
                },
            ],
            secrets: vec![SecretSpec {
                key: "DB_PASSWORD".into(),
                length: 32,
                format: SecretFormat::Hex,
            }],
        }
    }

    #[test]
    fn projection_excludes_sensitive_defaults_rules_environment_and_secrets() {
        let template = template();
        let json = serde_json::to_value(template.setup_review().unwrap()).unwrap();
        let text = json.to_string();
        for forbidden in ["private-default", "DB_PASSWORD", "environment", "compose"] {
            assert!(!text.contains(forbidden), "review disclosed {forbidden}");
        }
        assert_eq!(json["generated_credential_count"], 1);
        assert_eq!(json["fields"][0]["control"], "password");
        assert_eq!(json["fields"][0]["has_default"], true);
        assert_eq!(json["fields"][1]["default"], "My notes");
        // Existing setup semantics preserve defaults for missing or blank
        // answers. A UI must describe that behavior for masked defaults.
        assert_eq!(
            template.accept_answers(&Default::default()).unwrap()["API_KEY"],
            "private-default"
        );
        assert_eq!(
            template
                .accept_answers(&[("API_KEY".into(), String::new())].into())
                .unwrap()["API_KEY"],
            "private-default"
        );
    }

    #[test]
    fn review_requires_a_valid_plan_and_unknown_answers_are_rejected() {
        let mut template = template();
        assert!(template
            .accept_answers(&[("UNKNOWN".into(), "private-answer".into())].into())
            .is_err());
        template.plan.services[0].depends_on.push("missing".into());
        assert!(template.setup_review().is_err());
    }
}
