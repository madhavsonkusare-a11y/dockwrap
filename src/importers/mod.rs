//! Read-only importers for upstream deployment definitions.
//!
//! Each importer maps one upstream format onto a [`crate::plan::DeploymentPlan`]
//! and reports what it could not express. Importers never install, never write,
//! and never promote an app to a reviewed recipe: an imported plan is a
//! candidate for review, not a shipped one.
pub mod caprover;
pub mod runtipi;

use crate::plan::DeploymentPlan;
use crate::setup::PlanTemplate;

/// Why an app could not be imported, in the terms that decide who fixes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Limitation {
    /// Deliberately not expressible: accepting it would weaken the safety
    /// rules reviewed recipes are held to.
    Refused { feature: String, detail: String },
    /// The plan model has no field for this yet.
    NotModelled { feature: String, detail: String },
    /// Needs the typed setup fields and generated secrets that phase 2 has not
    /// built, so the value cannot be supplied.
    NeedsInput { feature: String, detail: String },
}
impl Limitation {
    pub fn feature(&self) -> &str {
        match self {
            Self::Refused { feature, .. }
            | Self::NotModelled { feature, .. }
            | Self::NeedsInput { feature, .. } => feature,
        }
    }
    pub fn category(&self) -> &'static str {
        match self {
            Self::Refused { .. } => "refused",
            Self::NotModelled { .. } => "not modelled",
            Self::NeedsInput { .. } => "needs input",
        }
    }
    pub fn detail(&self) -> &str {
        match self {
            Self::Refused { detail, .. }
            | Self::NotModelled { detail, .. }
            | Self::NeedsInput { detail, .. } => detail,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportOutcome {
    pub id: String,
    /// Present only when every part of the definition could be expressed,
    /// including a source for every placeholder.
    pub template: Option<PlanTemplate>,
    pub limitations: Vec<Limitation>,
}
impl ImportOutcome {
    pub fn is_importable(&self) -> bool {
        self.template.is_some()
    }
    /// The unresolved plan, for reporting shape without answering any field.
    pub fn plan(&self) -> Option<&DeploymentPlan> {
        self.template.as_ref().map(|template| &template.plan)
    }
}

pub(crate) fn refused(feature: &str, detail: impl Into<String>) -> Limitation {
    Limitation::Refused {
        feature: feature.to_owned(),
        detail: detail.into(),
    }
}
pub(crate) fn not_modelled(feature: &str, detail: impl Into<String>) -> Limitation {
    Limitation::NotModelled {
        feature: feature.to_owned(),
        detail: detail.into(),
    }
}
pub(crate) fn needs_input(feature: &str, detail: impl Into<String>) -> Limitation {
    Limitation::NeedsInput {
        feature: feature.to_owned(),
        detail: detail.into(),
    }
}

/// Where to publish a container port.
///
/// An unprivileged port is kept as it is, so the address a person sees matches
/// what upstream documents. A privileged one this computer may not bind gets
/// the conventional offset — 80 becomes 8080, 443 becomes 8443 — which always
/// lands inside the unprivileged range because only ports below 1024 take it.
///
/// Either way this is a preference, not a promise: the installer moves off it
/// when the port is already taken.
pub(crate) fn preferred_host_port(container: u16) -> u16 {
    if container >= 1024 {
        container
    } else {
        container.saturating_add(8000).max(1024)
    }
}

/// Read a Compose-style `command` or `entrypoint`, keeping the form upstream
/// used. A bare string is split the way a shell would split it and a list is
/// not, so collapsing the two would change what the container runs.
pub(crate) fn plan_args(value: &serde_json::Value) -> Option<crate::plan::PlanArgs> {
    match value {
        serde_json::Value::String(line) => Some(crate::plan::PlanArgs::Shell(line.clone())),
        serde_json::Value::Array(parts) => parts
            .iter()
            .map(|part| part.as_str().map(str::to_owned))
            .collect::<Option<Vec<String>>>()
            .map(crate::plan::PlanArgs::Exec),
        _ => None,
    }
}

/// Preserve startup versus health ordering; refuse completion jobs and optional
/// dependencies until their lifecycle semantics are implemented.
pub(crate) fn dependencies(
    value: Option<&serde_json::Value>,
) -> Result<(Vec<String>, std::collections::BTreeSet<String>), String> {
    use serde_json::Value;
    let mut names = Vec::new();
    let mut healthy = std::collections::BTreeSet::new();
    match value {
        None => {}
        Some(Value::Array(values)) => {
            for value in values {
                names.push(
                    value
                        .as_str()
                        .ok_or("Dependency name must be text")?
                        .to_owned(),
                );
            }
        }
        Some(Value::Object(values)) => {
            for (name, options) in values {
                let options = options
                    .as_object()
                    .ok_or("Dependency options must be an object")?;
                if options.keys().any(|k| k != "condition") {
                    return Err("Unsupported dependency option".into());
                }
                match options.get("condition") {
                    None => {}
                    Some(Value::String(s)) if s == "service_started" => {}
                    Some(Value::String(s)) if s == "service_healthy" => {
                        healthy.insert(name.clone());
                    }
                    _ => return Err("Unsupported dependency condition".into()),
                }
                names.push(name.clone());
            }
        }
        _ => return Err("Unsupported dependency declaration".into()),
    }
    let unique: std::collections::BTreeSet<_> = names.iter().collect();
    if unique.len() != names.len() {
        return Err("Duplicate dependency".into());
    }
    Ok((names, healthy))
}
