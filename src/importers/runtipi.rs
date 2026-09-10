//! Read-only importer for Runtipi app-store definitions.
//!
//! Runtipi already stores a normalized `docker-compose.json` rather than raw
//! Compose, which makes it the cheapest upstream source to map onto a
//! [`DeploymentPlan`]. This module only *reads*: it produces a plan when the
//! definition is fully expressible, and otherwise reports precisely what it
//! could not express. Nothing here installs anything, and an import is never
//! promoted to a reviewed recipe automatically.
//!
//! The report is the point. An app that cannot be imported is more useful as a
//! named limitation than as a silently skipped entry, because the limitations
//! are the work queue for the next phases.
use super::{
    needs_input, not_modelled, plan_args, preferred_host_port, refused, ImportOutcome, Limitation,
};
use crate::plan::{DeploymentPlan, PlanMount, PlanOverrides, PlanService, PublishedPort};
use crate::setup::{self, FieldKind, PlanTemplate, SecretSpec, SetupField};
use serde_json::Value;
use std::collections::BTreeMap;

/// Runtipi's placeholder for the directory it gives an app for its own data.
/// It is the only host path that maps onto a managed project directory.
const APP_DATA_DIR: &str = "${APP_DATA_DIR}/";

/// Host files an app mounts only to read the machine's clock setting.
///
/// These are the single most common reason a definition is refused, and they
/// are not a privilege request: the app wants to show local time. Mounting
/// host files to achieve that is a Linux-server habit that does not survive
/// the trip to Docker Desktop on Windows anyway, where the host filesystem is
/// not the daemon's filesystem. The supported mechanism is `TZ`, which this
/// importer already models, so the mount is dropped and a time zone answer is
/// ensured in its place.
const CLOCK_FILES: &[&str] = &["/etc/localtime", "/etc/timezone"];

/// Settings this project will not express, on purpose. Accepting them would
/// hand a container privileges the reviewed recipes deliberately refuse.
const REFUSED_KEYS: &[(&str, &str)] = &[
    ("privileged", "runs the container with full host privileges"),
    ("capAdd", "adds Linux capabilities"),
    ("capDrop", "changes Linux capabilities"),
    ("devices", "passes host devices into the container"),
    ("securityOpt", "changes the container security profile"),
    ("sysctls", "sets kernel parameters"),
    ("pid", "shares a process namespace with the host"),
    ("networkMode", "bypasses the isolated app network"),
];

/// Settings the plan model has no field for yet. These are additions to make,
/// not things to refuse.
const NOT_MODELLED_KEYS: &[(&str, &str)] = &[
    ("user", "runs as a specific user"),
    ("addPorts", "publishes additional ports"),
    ("extraLabels", "sets container labels"),
    ("logging", "configures a logging driver"),
    ("deploy", "sets deployment resources"),
    ("tty", "allocates a TTY"),
    ("stdinOpen", "keeps stdin open"),
    ("stopGracePeriod", "sets a stop grace period"),
    ("stopSignal", "sets a stop signal"),
    ("shmSize", "sets shared memory size"),
    ("workingDir", "sets a working directory"),
    ("ulimits", "sets resource limits"),
];

/// Values Runtipi's platform supplies at runtime that this project can supply
/// too, because a reviewed install is always loopback HTTP on a known port.
/// Anything not listed here is reported rather than guessed.
fn platform_values() -> BTreeMap<&'static str, String> {
    let mut values = BTreeMap::new();
    // Always http on loopback, whatever port is taken.
    values.insert("APP_PROTOCOL", "http".to_owned());
    // The port is not settled until installation, so these hand the address
    // on as a placeholder the installer fills rather than a guess made here.
    values.insert("APP_PORT", format!("${{{}}}", setup::PLATFORM_PORT));
    for key in ["APP_DOMAIN", "APP_HOST", "LOCAL_DOMAIN", "APP_LOCAL_DOMAIN"] {
        values.insert(key, format!("${{{}}}", setup::PLATFORM_HOST));
    }
    values
}

/// What one app's `form_fields` declare: values to ask for, values to generate,
/// and anything about them this project cannot honour.
struct DeclaredInputs {
    fields: Vec<SetupField>,
    secrets: Vec<SecretSpec>,
    limitations: Vec<Limitation>,
}

/// Read Runtipi's `form_fields` into typed fields and generated secrets.
fn read_form_fields(config: &str) -> Result<DeclaredInputs, String> {
    let root: Value = serde_json::from_str(config)
        .map_err(|error| format!("config is not valid JSON: {error}"))?;
    let mut fields = Vec::new();
    let mut secrets = Vec::new();
    let mut limitations = Vec::new();
    for entry in root
        .get("form_fields")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let key = entry
            .get("env_variable")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        if key.is_empty() {
            continue;
        }
        let kind_name = entry.get("type").and_then(Value::as_str).unwrap_or("text");
        let label = entry
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or(&key)
            .to_owned();
        let required = entry
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let min = entry.get("min").and_then(Value::as_i64);
        let max = entry.get("max").and_then(Value::as_i64);

        if entry.get("regex").is_some() {
            // Dropping a pattern silently would let a value through that the
            // app itself rejects, which reads as a broken install.
            limitations.push(not_modelled(
                "field pattern",
                format!("{key} is validated by a regular expression"),
            ));
        }
        if kind_name == "random" {
            let length = max.or(min).unwrap_or(32);
            let length = usize::try_from(length).unwrap_or(32).clamp(16, 256);
            secrets.push(SecretSpec {
                key,
                length,
                format: crate::setup::SecretFormat::Alphanumeric,
            });
            continue;
        }

        let default = match entry.get("default") {
            Some(Value::String(value)) => Some(value.clone()),
            Some(Value::Bool(value)) => Some(value.to_string()),
            Some(Value::Number(value)) => Some(value.to_string()),
            _ => None,
        };
        let options: Vec<String> = entry
            .get("options")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(|option| {
                        option
                            .get("value")
                            .and_then(Value::as_str)
                            .or_else(|| option.as_str())
                            .map(str::to_owned)
                    })
                    .collect()
            })
            .unwrap_or_default();

        let kind = if !options.is_empty() {
            FieldKind::Choice { options }
        } else {
            match kind_name {
                "boolean" => FieldKind::Boolean,
                "number" => FieldKind::Number {
                    min: min.unwrap_or(0),
                    max: max.unwrap_or(i64::from(u32::MAX)),
                },
                _ => FieldKind::Text {
                    min_len: usize::try_from(min.unwrap_or(0)).unwrap_or(0),
                    max_len: usize::try_from(max.unwrap_or(255)).unwrap_or(255).max(1),
                },
            }
        };
        fields.push(SetupField {
            key,
            label,
            kind,
            required,
            default,
            // Runtipi's `password` type is a credential the user supplies.
            sensitive: kind_name == "password",
        });
    }
    Ok(DeclaredInputs {
        fields,
        secrets,
        limitations,
    })
}

/// Map one Runtipi `docker-compose.json` onto a plan, or explain why not.
///
/// `id` is the app-store directory name. Errors are reserved for a definition
/// that cannot be read at all; anything readable produces an outcome, because
/// a described limitation is more useful than a dropped app.
pub fn import(id: &str, definition: &str, config: Option<&str>) -> Result<ImportOutcome, String> {
    let root: Value = serde_json::from_str(definition)
        .map_err(|error| format!("{id}: definition is not valid JSON: {error}"))?;
    let mut limitations = Vec::new();
    // Set when a service asked for the host clock, so the time zone answer is
    // offered in place of the mount that was dropped.
    let mut wants_clock = false;
    let declared = match config {
        Some(config) => read_form_fields(config).map_err(|error| format!("{id}: {error}"))?,
        None => DeclaredInputs {
            fields: Vec::new(),
            secrets: Vec::new(),
            limitations: Vec::new(),
        },
    };
    let (mut fields, secrets) = (declared.fields, declared.secrets);
    limitations.extend(declared.limitations);

    if root.get("overrides").is_some() {
        limitations.push(not_modelled(
            "overrides",
            "declares architecture-specific overrides",
        ));
    }
    let Some(services) = root.get("services").and_then(Value::as_array) else {
        return Err(format!("{id}: definition has no services array"));
    };
    if services.is_empty() {
        return Err(format!("{id}: definition declares no services"));
    }

    let mut planned = Vec::new();
    let mut main_seen = 0;
    for service in services {
        let name = service
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let image = service
            .get("image")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();

        for (key, detail) in REFUSED_KEYS {
            if service.get(*key).is_some() {
                limitations.push(refused(key, format!("{name} {detail}")));
            }
        }
        for (key, detail) in NOT_MODELLED_KEYS {
            if service.get(*key).is_some() {
                limitations.push(not_modelled(key, format!("{name} {detail}")));
            }
        }

        let is_main = service
            .get("isMain")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let published = match (is_main, service.get("internalPort").and_then(Value::as_u64)) {
            (true, Some(port)) if (1024..=u64::from(u16::MAX)).contains(&port) => {
                main_seen += 1;
                #[allow(clippy::cast_possible_truncation)]
                Some(PublishedPort {
                    host: port as u16,
                    container: port as u16,
                })
            }
            // Runtipi publishes on the container's own port, which a
            // privileged port cannot be. The container keeps the port it
            // actually listens on and the host gets the conventional offset
            // (80 becomes 8080, 443 becomes 8443). That is a preference, not a
            // promise: the installer moves off it when it is already taken.
            (true, Some(port)) if (1..1024).contains(&port) => {
                main_seen += 1;
                #[allow(clippy::cast_possible_truncation)]
                let container = port as u16;
                Some(PublishedPort {
                    host: preferred_host_port(container),
                    container,
                })
            }
            (true, _) => {
                main_seen += 1;
                limitations.push(not_modelled(
                    "internalPort",
                    format!("{name} is the main service but declares no usable port"),
                ));
                None
            }
            (false, _) => None,
        };

        let mut environment = Vec::new();
        for entry in service
            .get("environment")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default()
        {
            let key = entry.get("key").and_then(Value::as_str).unwrap_or_default();
            let value = match entry.get("value") {
                Some(Value::String(value)) => value.clone(),
                Some(Value::Number(number)) => number.to_string(),
                Some(Value::Bool(flag)) => flag.to_string(),
                _ => String::new(),
            };
            environment.push((key.to_owned(), value));
        }

        let mut mounts = Vec::new();
        for volume in service
            .get("volumes")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default()
        {
            let host = volume
                .get("hostPath")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let container = volume
                .get("containerPath")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            // A read-only mount only ever removes write access, so it is
            // carried through rather than blocking the import. The host path
            // check below still decides whether the mount is allowed at all.
            let read_only = volume.get("readOnly").and_then(Value::as_bool) == Some(true);
            if CLOCK_FILES.contains(&host) {
                // Dropped rather than refused, and replaced by the mechanism
                // that actually works here. The environment entry is what
                // makes the substitution real: a time zone field nothing
                // references would be dropped as inert, leaving the app in UTC
                // with no way to change it.
                wants_clock = true;
                if !environment.iter().any(|(key, _)| key == "TZ") {
                    environment.push(("TZ".to_owned(), "${TZ}".to_owned()));
                }
                continue;
            }
            match host.strip_prefix(APP_DATA_DIR) {
                Some(relative) if !relative.is_empty() => mounts.push(PlanMount::Directory {
                    source: relative.trim_end_matches('/').to_owned(),
                    target: container,
                    read_only,
                }),
                _ => limitations.push(refused(
                    "host path",
                    format!("{name} mounts {host:?}, which is outside the app's own storage"),
                )),
            }
        }

        let (depends_on, healthy_dependencies) = super::dependencies(service.get("dependsOn"))
            .unwrap_or_else(|reason| {
                limitations.push(not_modelled(
                    "dependsOn condition",
                    format!("{name}: {reason}"),
                ));
                Default::default()
            });

        // Carried through rather than dropped, for the same reason as every
        // other upstream constraint: leaving it out changes what starts.
        let mut overrides = PlanOverrides {
            healthy_dependencies,
            ..Default::default()
        };
        if let Some(value) = service.get("healthCheck") {
            let check = (|| {
                let object = value.as_object().ok_or("Health check must be an object")?;
                let mut normalized = serde_json::Map::new();
                for (key, value) in object {
                    let mapped = match key.as_str() {
                        "startPeriod" => "start_period",
                        "startInterval" => "start_interval",
                        "test" | "interval" | "timeout" | "retries" | "disable" => key,
                        _ => return Err("Unsupported Runtipi health check property".into()),
                    };
                    normalized.insert(mapped.to_owned(), value.clone());
                }
                crate::plan::PlanHealthcheck::from_compose(&Value::Object(normalized))
            })();
            match check {
                Ok(check) => overrides.healthcheck = Some(check),
                Err(reason) => {
                    limitations.push(not_modelled("healthCheck", format!("{name}: {reason}")))
                }
            }
        }
        for (field, value) in [
            ("command", service.get("command")),
            ("entrypoint", service.get("entrypoint")),
        ] {
            let Some(value) = value else { continue };
            match plan_args(value) {
                Some(args) if field == "command" => overrides.command = Some(args),
                Some(args) => overrides.entrypoint = Some(args),
                None => limitations.push(not_modelled(
                    field,
                    format!("{name} declares a {field} this importer cannot read"),
                )),
            }
        }
        if let Some(value) = service.get("hostname") {
            match value.as_str() {
                Some(hostname) => overrides.hostname = Some(hostname.to_owned()),
                None => limitations.push(not_modelled(
                    "hostname",
                    format!("{name} declares a hostname this importer cannot read"),
                )),
            }
        }

        planned.push(PlanService {
            name,
            image,
            environment,
            published,
            mounts,
            depends_on,
            overrides,
        });
    }

    if main_seen != 1 {
        limitations.push(not_modelled(
            "main service",
            format!("{main_seen} services are marked as the main one"),
        ));
    }

    // Fill the values Runtipi's platform would have supplied, then check that
    // every remaining placeholder has a declared field or secret behind it.
    let platform = platform_values();
    // Runtipi supplies TZ globally. Local Store has no global timezone setting,
    // so expose an optional per-app answer instead of guessing the host zone.
    // Preserve an upstream declaration (including its default) when present.
    let uses_timezone = wants_clock
        || planned
            .iter()
            .flat_map(|service| &service.environment)
            .any(|(_, value)| placeholder_keys(value).iter().any(|key| key == "TZ"));
    if uses_timezone
        && !fields.iter().any(|field| field.key == "TZ")
        && !secrets.iter().any(|secret| secret.key == "TZ")
    {
        fields.push(SetupField {
            key: "TZ".into(),
            label: "Time zone (for example, Asia/Kolkata)".into(),
            kind: FieldKind::Text {
                min_len: 1,
                max_len: 128,
            },
            required: false,
            default: Some("UTC".into()),
            sensitive: false,
        });
    }
    for service in &mut planned {
        for (key, value) in &mut service.environment {
            *value = fill_platform_values(value, &platform);
            for placeholder in placeholder_keys(value) {
                let declared = setup::is_platform_key(&placeholder)
                    || fields.iter().any(|field| field.key == placeholder)
                    || secrets.iter().any(|secret| secret.key == placeholder);
                if !declared {
                    limitations.push(needs_input(
                        "platform placeholder",
                        format!("{key} expects {placeholder}, which this project does not supply"),
                    ));
                }
            }
        }
    }

    let planned_environment: Vec<String> = planned
        .iter()
        .flat_map(|service| &service.environment)
        .map(|(_, value)| value.clone())
        .collect();

    // Only build a template when nothing was dropped. One assembled from a
    // partially understood definition would look installable and would not be.
    let template = if limitations.is_empty() {
        let candidate = PlanTemplate {
            plan: DeploymentPlan {
                id: id.to_owned(),
                services: planned,
                named_volumes: Vec::new(),
            },
            // Only keep what the definition actually uses; Runtipi declares
            // fields for optional features an app may never reference.
            fields: fields
                .into_iter()
                .filter(|field| uses_placeholder(&planned_environment, &field.key))
                .collect(),
            secrets: secrets
                .into_iter()
                .filter(|secret| uses_placeholder(&planned_environment, &secret.key))
                .collect(),
        };
        match candidate.validate() {
            Ok(()) => Some(candidate),
            Err(reason) => {
                limitations.push(refused("plan policy", reason));
                None
            }
        }
    } else {
        None
    };

    limitations.sort_by(|a, b| {
        a.feature()
            .cmp(b.feature())
            .then(a.detail().cmp(b.detail()))
    });
    limitations.dedup_by(|a, b| a.feature() == b.feature() && a.detail() == b.detail());
    Ok(ImportOutcome {
        id: id.to_owned(),
        template,
        limitations,
    })
}

/// Replace the platform values this project can supply for itself.
fn fill_platform_values(value: &str, platform: &BTreeMap<&'static str, String>) -> String {
    let mut out = value.to_owned();
    for (key, replacement) in platform {
        out = out.replace(&format!("${{{key}}}"), replacement);
        // Runtipi also writes `${KEY:-fallback}`; our value replaces both.
        while let Some(start) = out.find(&format!("${{{key}:-")) {
            let Some(end) = out[start..].find('}') else {
                break;
            };
            out.replace_range(start..start + end + 1, replacement);
        }
    }
    out
}

/// Placeholder names still present in a value.
fn placeholder_keys(value: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut rest = value;
    while let Some(start) = rest.find("${") {
        let tail = &rest[start + 2..];
        let Some(end) = tail.find('}') else { break };
        let inner = &tail[..end];
        let name = inner.split(":-").next().unwrap_or(inner);
        if !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        {
            keys.push(name.to_owned());
        }
        rest = &tail[end + 1..];
    }
    keys
}

fn uses_placeholder(environment: &[String], key: &str) -> bool {
    environment
        .iter()
        .any(|value| placeholder_keys(value).iter().any(|found| found == key))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE: &str = r#"{
      "schemaVersion": 1,
      "services": [{
        "name": "notes",
        "image": "example/notes:1.4.2",
        "isMain": true,
        "internalPort": 8080,
        "environment": [{"key": "TZ", "value": "UTC"}],
        "volumes": [{"hostPath": "${APP_DATA_DIR}/data", "containerPath": "/data"}]
      }]
    }"#;

    /// The commonest reason a Runtipi definition was refused was not a
    /// privilege request at all: the app mounted the host clock so it could
    /// show local time. Dropping that mount and asking for a time zone instead
    /// is the same intent through a mechanism that works on Windows, where the
    /// host filesystem is not the daemon's filesystem.
    #[test]
    fn mounting_the_host_clock_asks_for_a_time_zone_instead_of_being_refused() {
        let definition = r#"{"services":[{"name":"app","image":"example/app:1.0","isMain":true,"internalPort":8080,"volumes":[{"hostPath":"/etc/localtime","containerPath":"/etc/localtime","readOnly":true},{"hostPath":"/etc/timezone","containerPath":"/etc/timezone","readOnly":true},{"hostPath":"${APP_DATA_DIR}/data","containerPath":"/data"}]}]}"#;
        let outcome = import("app", definition, None).expect("import should not fail");
        let template = outcome
            .template
            .expect("clock mounts must not block an import");

        // The clock mounts are gone; the app's own storage is untouched.
        let mounts = &template.plan.services[0].mounts;
        assert_eq!(mounts.len(), 1, "{mounts:?}");
        assert!(!outcome
            .limitations
            .iter()
            .any(|limit| limit.feature() == "host path"));

        // And the intent survives as an answer with a sane default.
        let zone = template
            .fields
            .iter()
            .find(|field| field.key == "TZ")
            .expect("a dropped clock mount must offer a time zone");
        assert!(!zone.required);
        assert_eq!(zone.default.as_deref(), Some("UTC"));
    }

    /// A host path that is not the clock stays refused. This is the guard that
    /// stops the exception above from becoming a general permission to mount
    /// host directories.
    #[test]
    fn any_other_host_path_is_still_refused() {
        for host in [
            "/var/run/docker.sock",
            "/var/log/auth.log",
            "${ROOT_FOLDER_HOST}/media",
        ] {
            let definition = format!(
                r#"{{"services":[{{"name":"app","image":"example/app:1.0","isMain":true,"internalPort":8080,"volumes":[{{"hostPath":"{host}","containerPath":"/x"}}]}}]}}"#
            );
            let outcome = import("app", &definition, None).expect("import should not fail");
            assert!(
                outcome
                    .limitations
                    .iter()
                    .any(|limit| limit.feature() == "host path"),
                "{host} was not refused"
            );
            assert!(outcome.template.is_none(), "{host} produced a template");
        }
    }

    #[test]
    fn a_plain_single_service_app_becomes_a_plan() {
        let outcome = import("notes", SIMPLE, None).unwrap();
        assert!(outcome.limitations.is_empty(), "{:?}", outcome.limitations);
        let plan = outcome.plan().expect("a plan").clone();
        assert_eq!(plan.id, "notes");
        assert_eq!(plan.data_directories(), vec!["data"]);
        let (service, port) = plan.published().expect("published endpoint");
        assert_eq!(service.name, "notes");
        assert_eq!(port.host, 8080);
        // It has to render, not merely construct.
        assert!(plan.to_compose().unwrap().contains("127.0.0.1:8080:8080"));
    }

    #[test]
    fn a_read_only_mount_is_carried_through_rather_than_blocking_the_import() {
        let definition = r#"{
          "services": [{
            "name": "notes", "image": "example/notes:1.4.2", "isMain": true, "internalPort": 8080,
            "volumes": [
              {"hostPath": "${APP_DATA_DIR}/data", "containerPath": "/data"},
              {"hostPath": "${APP_DATA_DIR}/config", "containerPath": "/etc/notes", "readOnly": true}
            ]
          }]
        }"#;
        let outcome = import("notes", definition, None).unwrap();
        assert!(outcome.limitations.is_empty(), "{:?}", outcome.limitations);
        let compose = outcome.plan().expect("a plan").to_compose().unwrap();
        assert!(compose.contains("- ./config:/etc/notes:ro"), "{compose}");
        // The writable mount must not pick the flag up on its way past.
        assert!(
            compose.contains(
                "- ./data:/data
"
            ),
            "{compose}"
        );
    }

    #[test]
    fn a_privileged_container_port_gets_an_unprivileged_host_port() {
        let definition = r#"{
          "services": [{
            "name": "web", "image": "example/web:1.0.0", "isMain": true, "internalPort": 80,
            "volumes": [{"hostPath": "${APP_DATA_DIR}/data", "containerPath": "/data"}]
          }]
        }"#;
        let outcome = import("web", definition, None).unwrap();
        assert!(outcome.limitations.is_empty(), "{:?}", outcome.limitations);
        let plan = outcome.plan().expect("a plan");
        let (_, port) = plan.published().expect("a published endpoint");
        // The container keeps the port it listens on; only the host side moves.
        assert_eq!(port.container, 80);
        assert_eq!(port.host, 8080);
        assert!(plan.to_compose().unwrap().contains("127.0.0.1:8080:80"));
    }

    #[test]
    fn a_port_outside_the_addressable_range_is_still_reported() {
        let definition = r#"{
          "services": [{
            "name": "web", "image": "example/web:1.0.0", "isMain": true, "internalPort": 70000
          }]
        }"#;
        let outcome = import("web", definition, None).unwrap();
        assert!(!outcome.is_importable());
        assert!(
            outcome
                .limitations
                .iter()
                .any(|limit| limit.feature() == "internalPort"),
            "{:?}",
            outcome.limitations
        );
    }

    #[test]
    fn a_privileged_or_escaping_definition_is_refused_and_named() {
        let definition = r#"{
          "services": [{
            "name": "tool", "image": "example/tool:2.0", "isMain": true, "internalPort": 9000,
            "privileged": true, "networkMode": "host",
            "volumes": [{"hostPath": "${ROOT_FOLDER_HOST}/etc", "containerPath": "/etc"}]
          }]
        }"#;
        let outcome = import("tool", definition, None).unwrap();
        assert!(!outcome.is_importable());
        let refused: Vec<_> = outcome
            .limitations
            .iter()
            .filter(|limit| limit.category() == "refused")
            .map(Limitation::feature)
            .collect();
        assert!(refused.contains(&"privileged"), "{refused:?}");
        assert!(refused.contains(&"networkMode"), "{refused:?}");
        assert!(refused.contains(&"host path"), "{refused:?}");
    }

    #[test]
    fn a_supplied_value_is_reported_rather_than_invented() {
        let definition = r#"{
          "services": [{
            "name": "app", "image": "example/app:3.1", "isMain": true, "internalPort": 8080,
            "environment": [
              {"key": "DB_PASSWORD", "value": "${DB_PASSWORD}"},
              {"key": "TZ", "value": "UTC"}
            ]
          }]
        }"#;
        let outcome = import("app", definition, None).unwrap();
        assert!(!outcome.is_importable());
        assert_eq!(
            outcome
                .limitations
                .iter()
                .filter(|limit| limit.category() == "needs input")
                .count(),
            1
        );
        // The placeholder is never carried through as a literal value.
        assert!(!format!("{outcome:?}").contains("\"${DB_PASSWORD}\""));
    }

    #[test]
    fn a_web_app_with_a_database_keeps_its_ordering_and_publishes_once() {
        let definition = r#"{
          "services": [
            {"name": "web", "image": "example/web:1.0.0", "isMain": true, "internalPort": 3000,
             "dependsOn": ["db"],
             "volumes": [{"hostPath": "${APP_DATA_DIR}/data", "containerPath": "/data"}]},
            {"name": "db", "image": "example/db:16.2",
             "volumes": [{"hostPath": "${APP_DATA_DIR}/db", "containerPath": "/var/lib/db"}]}
          ]
        }"#;
        let outcome = import("paired", definition, None).unwrap();
        assert!(outcome.is_importable(), "{:?}", outcome.limitations);
        let plan = outcome.plan().unwrap().clone();
        assert_eq!(plan.services.len(), 2);
        let rendered = plan.to_compose().unwrap();
        assert!(rendered.contains("depends_on:\n      - db"));
        // Exactly one endpoint reaches the host, and it is not the database.
        assert_eq!(rendered.matches("127.0.0.1:").count(), 1);
        assert!(rendered.contains("127.0.0.1:3000:3000"));
    }

    #[test]
    fn a_health_condition_and_probe_survive_import() {
        let definition = r#"{
          "services": [
            {"name": "web", "image": "example/web:1.0.0", "isMain": true, "internalPort": 3000,
             "dependsOn": {"db": {"condition": "service_healthy"}}},
            {"name": "db", "image": "example/db:16.2", "healthCheck": {"test":"pg_isready", "startPeriod":"10s", "interval":"2s"}}
          ]
        }"#;
        let outcome = import("conditional", definition, None).unwrap();
        assert!(outcome.is_importable(), "{:?}", outcome.limitations);
        let rendered = outcome.plan().unwrap().to_compose().unwrap();
        assert!(rendered.contains("condition: service_healthy"));
        assert_eq!(
            outcome.plan().unwrap().services[1]
                .overrides
                .healthcheck
                .as_ref()
                .unwrap()
                .start_period
                .as_deref(),
            Some("10s")
        );
        for replacement in ["service_completed_successfully", "unknown"] {
            let blocked = import(
                "conditional",
                &definition.replace("service_healthy", replacement),
                None,
            )
            .unwrap();
            assert!(blocked.template.is_none());
        }
        let cyclic = definition.replace(
            "\"healthCheck\":",
            "\"dependsOn\":[\"web\"], \"healthCheck\":",
        );
        assert!(import("conditional", &cyclic, None)
            .unwrap()
            .template
            .is_none());
    }

    const WITH_INPUTS: &str = r#"{
      "services": [{
        "name": "app", "image": "example/app:3.1", "isMain": true, "internalPort": 8080,
        "environment": [
          {"key": "DB_PASSWORD", "value": "${DB_PASSWORD}"},
          {"key": "ADMIN_EMAIL", "value": "${ADMIN_EMAIL}"},
          {"key": "SIGNUPS", "value": "${ALLOW_SIGNUPS}"},
          {"key": "SITE", "value": "${APP_PROTOCOL:-http}://${APP_DOMAIN}"}
        ]
      }]
    }"#;
    const CONFIG: &str = r#"{
      "id": "app",
      "form_fields": [
        {"type": "random", "label": "Database password", "min": 32, "max": 32, "env_variable": "DB_PASSWORD"},
        {"type": "email", "label": "Admin email", "required": true, "env_variable": "ADMIN_EMAIL"},
        {"type": "boolean", "label": "Allow signups", "default": false, "env_variable": "ALLOW_SIGNUPS"},
        {"type": "password", "label": "API key", "env_variable": "UNUSED_KEY"}
      ]
    }"#;

    #[test]
    fn declared_form_fields_turn_placeholders_into_fields_and_secrets() {
        let outcome = import("app", WITH_INPUTS, Some(CONFIG)).unwrap();
        assert!(outcome.is_importable(), "{:?}", outcome.limitations);
        let template = outcome.template.expect("a template");

        // `random` becomes a generated secret, not something to ask for.
        assert_eq!(template.secrets.len(), 1);
        assert_eq!(template.secrets[0].key, "DB_PASSWORD");
        assert_eq!(template.secrets[0].length, 32);

        // A field the definition never references is not carried along.
        let keys: Vec<&str> = template.fields.iter().map(|f| f.key.as_str()).collect();
        assert_eq!(keys, vec!["ADMIN_EMAIL", "ALLOW_SIGNUPS"]);
        assert!(template.fields[0].required);
        assert_eq!(template.fields[1].kind, crate::setup::FieldKind::Boolean);
        assert_eq!(template.fields[1].default.as_deref(), Some("false"));

        // The app's own address is handed on as a placeholder, not baked: the
        // port is not settled until the install chooses one.
        let site = &template.plan.services[0].environment[3].1;
        assert_eq!(site, "http://${LOCAL_STORE_HOST}");

        // And the whole thing resolves into an installable plan once the
        // installer has supplied the address.
        let secrets = template.generate_secrets().unwrap();
        let answers = [("ADMIN_EMAIL".to_owned(), "me@example.com".to_owned())]
            .into_iter()
            .collect();
        let mut filled = template.clone();
        crate::setup::fill_platform_values(&mut filled.plan, 8080);
        let plan = filled.resolve(&answers, &secrets).unwrap();
        let compose = plan.to_compose().unwrap();
        assert!(!compose.contains("${"), "{compose}");
        assert!(compose.contains("http://localhost:8080"), "{compose}");
    }

    #[test]
    fn a_password_field_is_marked_so_an_interface_can_mask_it() {
        let definition = r#"{
          "services": [{"name": "app", "image": "example/app:1.0", "isMain": true,
            "internalPort": 8080,
            "environment": [{"key": "API_KEY", "value": "${API_KEY}"}]}]
        }"#;
        let config = r#"{"form_fields": [
          {"type": "password", "label": "API key", "required": true, "env_variable": "API_KEY"}
        ]}"#;
        let template = import("app", definition, Some(config))
            .unwrap()
            .template
            .expect("a template");
        assert!(template.fields[0].sensitive, "a password must be marked");
    }

    #[test]
    fn a_placeholder_with_no_declared_field_is_still_reported() {
        // `INTERNAL_IP` is a Runtipi platform value this project does not have.
        let definition = r#"{
          "services": [{"name": "app", "image": "example/app:1.0", "isMain": true,
            "internalPort": 8080,
            "environment": [{"key": "HOST", "value": "${INTERNAL_IP}"}]}]
        }"#;
        let outcome = import("app", definition, Some(r#"{"form_fields": []}"#)).unwrap();
        assert!(!outcome.is_importable());
        assert!(outcome
            .limitations
            .iter()
            .any(|limit| limit.feature() == "platform placeholder"));
    }

    #[test]
    fn an_unreadable_definition_is_an_error_not_a_silent_skip() {
        assert!(import("broken", "{ not json", None).is_err());
        assert!(import("empty", r#"{"services": []}"#, None).is_err());
        assert!(import("missing", r#"{"schemaVersion": 1}"#, None).is_err());
    }

    #[test]
    fn timezone_is_optional_overridable_and_preserves_upstream_declarations() {
        let definition = r#"{"services":[{"name":"app","image":"example/app:1.0","isMain":true,"internalPort":8080,"environment":[{"key":"TZ","value":"${TZ}"},{"key":"OTHER","value":"${TZ}"}]}]}"#;
        let template = import("app", definition, None).unwrap().template.unwrap();
        assert_eq!(template.fields.len(), 1);
        assert!(!template.fields[0].required);
        let plan = template
            .resolve(&BTreeMap::new(), &BTreeMap::new())
            .unwrap();
        assert_eq!(plan.services[0].environment[0].1, "UTC");
        let answers = BTreeMap::from([("TZ".into(), "Asia/Kolkata".into())]);
        let plan = template.resolve(&answers, &BTreeMap::new()).unwrap();
        assert!(plan.services[0]
            .environment
            .iter()
            .all(|(_, value)| value == "Asia/Kolkata"));
        let config = r#"{"form_fields":[{"type":"text","env_variable":"TZ","label":"Upstream zone","default":"Europe/Paris"}]}"#;
        let declared = import("app", definition, Some(config))
            .unwrap()
            .template
            .unwrap();
        assert_eq!(declared.fields.len(), 1);
        assert_eq!(declared.fields[0].label, "Upstream zone");
        assert_eq!(declared.fields[0].default.as_deref(), Some("Europe/Paris"));
        let literal = definition.replace("${TZ}", "UTC");
        assert!(import("app", &literal, None)
            .unwrap()
            .template
            .unwrap()
            .fields
            .is_empty());
    }
}
