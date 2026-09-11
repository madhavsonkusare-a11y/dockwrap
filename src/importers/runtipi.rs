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

/// Runtipi's placeholder for the machine's own storage root.
///
/// On a Runtipi box this is the install directory, and `media/data/<kind>` is
/// a library several apps share. Local Store has no such root, and inventing
/// one would put a person's music inside an application's data directory. What
/// it has instead is a folder the person picks, so a path *under* this
/// placeholder becomes a question rather than a refusal.
///
/// The bare placeholder stays refused: that is the whole storage root, not a
/// folder anybody meant to share.
const ROOT_FOLDER_HOST: &str = "${ROOT_FOLDER_HOST}/";

/// Container paths a shared folder may never be mounted over.
///
/// These belong to the image, not to the person. Replacing them changes what
/// runs rather than what it can read.
fn container_path_is_system(target: &str) -> bool {
    let cleaned = target.trim_end_matches('/');
    if cleaned.is_empty() || cleaned == "/" {
        return true;
    }
    const SYSTEM: &[&str] = &[
        "/etc", "/usr", "/bin", "/sbin", "/lib", "/lib64", "/boot", "/dev", "/proc", "/sys",
        "/run", "/var/run",
    ];
    SYSTEM
        .iter()
        .any(|system| cleaned == *system || cleaned.starts_with(&format!("{system}/")))
}

/// A setup key and a human label for a shared library path.
///
/// `media/data/music` becomes `FOLDER_MUSIC` and "Music folder". The key has
/// to be NAME_LIKE_THIS because it is also an environment-style placeholder,
/// and it has to be derived from the path so two mounts of the same library in
/// one definition ask the person once rather than twice.
fn shared_folder_field(subpath: &str) -> Option<(String, String)> {
    let cleaned = subpath.trim_matches('/');
    if cleaned.is_empty() || cleaned.contains("..") {
        return None;
    }
    // Only the shared library. Everything else under this placeholder is
    // Runtipi's own installation — its `etc`, its state, its other apps — and
    // none of that is a folder a person meant to hand over.
    if cleaned != "media" && !cleaned.starts_with("media/") {
        return None;
    }
    let parts: Vec<&str> = cleaned
        .split('/')
        // `media` and `data` are Runtipi's own scaffolding, not what the
        // folder is. Dropping them turns `media/data/music` into "Music".
        .filter(|part| !matches!(*part, "media" | "data") && !part.is_empty())
        .collect();
    let named: Vec<&str> = if parts.is_empty() {
        vec!["media"]
    } else {
        parts
    };
    let key: String = named
        .join("_")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect();
    if key.is_empty() || !key.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return None;
    }
    let mut label = named.join(" ").replace(['-', '_'], " ");
    if let Some(first) = label.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    Some((format!("FOLDER_{key}"), format!("{label} folder")))
}

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
    ("extraLabels", "sets container labels"),
    ("logging", "configures a logging driver"),
    ("deploy", "sets deployment resources"),
    ("tty", "allocates a TTY"),
    ("stdinOpen", "keeps stdin open"),
    ("stopGracePeriod", "sets a stop grace period"),
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
            // Runtipi reads `min` as bytes for base64 and as characters for
            // hex. Without an encoding the value stays alphanumeric, which is
            // what every approved app was proven with.
            let format = match entry.get("encoding").and_then(Value::as_str) {
                None => crate::setup::SecretFormat::Alphanumeric,
                Some("hex") => crate::setup::SecretFormat::Hex,
                Some("base64") => crate::setup::SecretFormat::Base64,
                Some(other) => {
                    limitations.push(not_modelled(
                        "secret encoding",
                        format!("{key} asks for a {other} value"),
                    ));
                    continue;
                }
            };
            secrets.push(SecretSpec {
                key,
                length,
                format,
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
/// The files Runtipi copies into an app's data folder on install, read from
/// the `data/` folder beside its definition.
///
/// Returned with the paths of any that could not be carried: seeds travel as
/// text inside a reviewed manifest, so a binary file — Calibre's starter
/// `metadata.db` — is reported rather than silently dropped.
pub fn read_seeds(app_folder: &std::path::Path) -> (Vec<crate::setup::SeedFile>, Vec<String>) {
    let mut seeds = Vec::new();
    let mut skipped = Vec::new();
    let mut pending = vec![app_folder.join("data")];
    while let Some(folder) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&folder) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let Ok(relative) = path.strip_prefix(app_folder) else {
                continue;
            };
            let relative = relative.to_string_lossy().replace('\\', "/");
            match std::fs::read(&path).map(String::from_utf8) {
                Ok(Ok(content)) => seeds.push(crate::setup::SeedFile {
                    path: relative,
                    content,
                }),
                _ => skipped.push(relative),
            }
        }
    }
    seeds.sort_by(|a, b| a.path.cmp(&b.path));
    skipped.sort();
    (seeds, skipped)
}

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

        // Runtipi's extra ports, which Local Store publishes on loopback as a
        // second address the app's own pages call: Sim's realtime socket,
        // Maxun's backend. One per service, TCP only, never the main one.
        let mut companion = None;
        if let Some(value) = service.get("addPorts") {
            match companion_port(value) {
                Ok(_) if is_main => limitations.push(not_modelled(
                    "addPorts",
                    format!("{name} is the main service; a second address belongs to another one"),
                )),
                Ok(port) => companion = Some(port),
                Err(reason) => {
                    limitations.push(not_modelled("addPorts", format!("{name} {reason}")))
                }
            }
        }

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
            if let Some(subpath) = host.strip_prefix(ROOT_FOLDER_HOST) {
                // Where it lands inside the container matters as much as where
                // it comes from. A person's folder mounted over the container's
                // own `/etc` or `/usr` is not a shared library, it is a way to
                // replace the software that is about to run.
                if container_path_is_system(&container) {
                    limitations.push(refused(
                        "host path",
                        format!("{name} mounts a folder over {container:?}, which is the container's own system"),
                    ));
                    continue;
                }
                match shared_folder_field(subpath) {
                    Some((key, label)) => {
                        // One question per library, however many services
                        // mount it.
                        if !fields.iter().any(|field: &SetupField| field.key == key) {
                            fields.push(SetupField {
                                key: key.clone(),
                                label,
                                kind: FieldKind::Folder { read_only },
                                required: true,
                                default: None,
                                sensitive: false,
                            });
                        }
                        mounts.push(PlanMount::Host {
                            source: format!("${{{key}}}"),
                            target: container,
                            read_only,
                        });
                    }
                    None => limitations.push(refused(
                        "host path",
                        format!(
                            "{name} mounts {host:?}, which names no folder a person could choose"
                        ),
                    )),
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

        let declared_dependencies =
            super::dependencies(service.get("dependsOn")).unwrap_or_else(|reason| {
                limitations.push(not_modelled(
                    "dependsOn condition",
                    format!("{name}: {reason}"),
                ));
                Default::default()
            });
        let depends_on = declared_dependencies.names;

        // Carried through rather than dropped, for the same reason as every
        // other upstream constraint: leaving it out changes what starts.
        let mut overrides = PlanOverrides {
            healthy_dependencies: declared_dependencies.healthy,
            completed_dependencies: declared_dependencies.completed,
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
        if let Some(value) = service.get("stopSignal") {
            match value.as_str() {
                Some(signal) => overrides.stop_signal = Some(signal.to_owned()),
                None => limitations.push(not_modelled(
                    "stopSignal",
                    format!("{name} declares a stop signal this importer cannot read"),
                )),
            }
        }

        planned.push(PlanService {
            name,
            image,
            digest: None,
            environment,
            companion,
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
    let companion_services: Vec<String> = planned
        .iter()
        .filter(|service| service.companion.is_some())
        .map(|service| service.name.clone())
        .collect();
    for service in &mut planned {
        for (key, value) in &mut service.environment {
            *value = fill_platform_values(value, &platform);
            for placeholder in placeholder_keys(value) {
                let declared = setup::is_platform_key(&placeholder)
                    || setup::companion_service(&placeholder).is_some_and(|wanted| {
                        companion_services
                            .iter()
                            .any(|name| setup::companion_suffix(name) == wanted)
                    })
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

    // Every host-mount source, so a folder answer counts as referenced.
    let planned_mount_sources: Vec<String> = planned
        .iter()
        .flat_map(|service| &service.mounts)
        .filter_map(|mount| match mount {
            PlanMount::Host { source, .. } => Some(source.clone()),
            _ => None,
        })
        .collect();

    // Only build a template when nothing was dropped. One assembled from a
    // partially understood definition would look installable and would not be.
    let template = if limitations.is_empty() {
        let candidate = PlanTemplate {
            first_start: None,
            seeds: Vec::new(),
            plan: DeploymentPlan {
                id: id.to_owned(),
                services: planned,
                named_volumes: Vec::new(),
            },
            // Only keep what the definition actually uses; Runtipi declares
            // fields for optional features an app may never reference. A
            // shared folder is referenced by a mount rather than by an
            // environment value, so both are consulted — checking only the
            // environment dropped the folder field and then refused the
            // template for referring to a key nothing declared.
            fields: fields
                .into_iter()
                .filter(|field| {
                    uses_placeholder(&planned_environment, &field.key)
                        || uses_placeholder(&planned_mount_sources, &field.key)
                })
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

/// Read one service's `addPorts` as a second loopback address.
fn companion_port(value: &Value) -> Result<PublishedPort, String> {
    let entries = value
        .as_array()
        .ok_or("declares addPorts that are not a list")?;
    let [entry] = entries.as_slice() else {
        return Err(format!(
            "declares {} extra ports; one is supported",
            entries.len()
        ));
    };
    let object = entry
        .as_object()
        .ok_or("declares an extra port that is not an object")?;
    for key in object.keys() {
        if !matches!(key.as_str(), "hostPort" | "containerPort" | "tcp") {
            return Err(format!("sets {key} on an extra port"));
        }
    }
    if object
        .get("tcp")
        .is_some_and(|tcp| tcp != &Value::Bool(true))
    {
        return Err("publishes an extra port that is not TCP".into());
    }
    let port = |key: &str| {
        object
            .get(key)
            .and_then(Value::as_u64)
            .and_then(|port| u16::try_from(port).ok())
            .filter(|port| *port > 0)
            .ok_or(format!("declares no usable {key}"))
    };
    let (host, container) = (port("hostPort")?, port("containerPort")?);
    Ok(PublishedPort {
        // A privileged host port is a preference the installer cannot take;
        // the same offset as the main port keeps it recognisable.
        host: if host < 1024 {
            preferred_host_port(host)
        } else {
            host
        },
        container,
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

    /// The libraries people actually keep outside an app: photos, music,
    /// books. Runtipi shares one directory between apps; Local Store has no
    /// such place, so it asks instead.
    #[test]
    fn a_shared_library_path_becomes_a_folder_a_person_chooses() {
        let definition = r#"{"services":[{"name":"app","image":"example/app:1.0","isMain":true,"internalPort":8080,"volumes":[{"hostPath":"${ROOT_FOLDER_HOST}/media/data/music","containerPath":"/music","readOnly":true},{"hostPath":"${APP_DATA_DIR}/data","containerPath":"/data"}]}]}"#;
        let outcome = import("app", definition, None).expect("import should not fail");
        let template = outcome.template.clone().unwrap_or_else(|| {
            panic!(
                "blocked: {:?}",
                outcome
                    .limitations
                    .iter()
                    .map(|l| (l.category(), l.feature()))
                    .collect::<Vec<_>>()
            )
        });

        let folder = template
            .fields
            .iter()
            .find(|field| field.key == "FOLDER_MUSIC")
            .expect("the music folder is asked for");
        assert_eq!(folder.label, "Music folder");
        assert!(folder.required);
        assert_eq!(folder.kind, FieldKind::Folder { read_only: true });

        // The mount refers to the answer, never to a path nobody chose.
        let mounts = &template.plan.services[0].mounts;
        assert!(mounts.contains(&PlanMount::Host {
            source: "${FOLDER_MUSIC}".into(),
            target: "/music".into(),
            read_only: true,
        }));
        assert!(mounts.iter().any(|mount| matches!(
            mount,
            PlanMount::Directory { source, .. } if source == "data"
        )));
    }

    /// Two services sharing one library is one question, not two.
    #[test]
    fn the_same_library_mounted_twice_is_asked_about_once() {
        let definition = r#"{"services":[{"name":"app","image":"example/app:1.0","isMain":true,"internalPort":8080,"volumes":[{"hostPath":"${ROOT_FOLDER_HOST}/media/data/books","containerPath":"/books"}]},{"name":"worker","image":"example/worker:1.0","volumes":[{"hostPath":"${ROOT_FOLDER_HOST}/media/data/books","containerPath":"/library"}]}]}"#;
        let outcome = import("app", definition, None).expect("import should not fail");
        let template = outcome.template.expect("this should import");
        assert_eq!(
            template
                .fields
                .iter()
                .filter(|field| field.key == "FOLDER_BOOKS")
                .count(),
            1
        );
        let targets: Vec<&str> = template
            .plan
            .services
            .iter()
            .flat_map(|service| &service.mounts)
            .filter_map(|mount| match mount {
                PlanMount::Host { target, .. } => Some(target.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(targets, vec!["/books", "/library"]);
    }

    /// Where a folder lands inside the container matters as much as where it
    /// came from: mounted over the container's own system it would replace the
    /// software rather than feed it.
    #[test]
    fn a_shared_folder_may_not_be_mounted_over_the_containers_own_system() {
        for target in ["/etc", "/usr/bin", "/", "/var/run"] {
            let definition = format!(
                r#"{{"services":[{{"name":"app","image":"example/app:1.0","isMain":true,"internalPort":8080,"volumes":[{{"hostPath":"${{ROOT_FOLDER_HOST}}/media/data/music","containerPath":"{target}"}}]}}]}}"#
            );
            let outcome = import("app", &definition, None).expect("import should not fail");
            assert!(
                outcome
                    .limitations
                    .iter()
                    .any(|limit| limit.feature() == "host path"),
                "{target} was allowed"
            );
            assert!(outcome.template.is_none(), "{target} produced a template");
        }
    }

    /// A host path that is not the clock stays refused. This is the guard that
    /// stops the exception above from becoming a general permission to mount
    /// host directories.
    #[test]
    fn any_other_host_path_is_still_refused() {
        for host in [
            "/var/run/docker.sock",
            "/var/log/auth.log",
            // The storage root itself, not a folder inside it.
            "${ROOT_FOLDER_HOST}",
            // Runtipi's own installation, which is not a shared library.
            "${ROOT_FOLDER_HOST}/etc",
            "${ROOT_FOLDER_HOST}/state",
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

    /// Postgres shuts down cleanly only on SIGINT. Penpot's definition says
    /// so, and refusing it cost Penpot its import.
    #[test]
    fn a_stop_signal_is_carried_through_and_a_bad_one_refused() {
        let definition = r#"{"services": [{"name": "db", "image": "postgres:15.14",
            "isMain": true, "internalPort": 5432, "stopSignal": "SIGINT"}]}"#;
        let outcome = import("db", definition, None).unwrap();
        assert!(outcome.limitations.is_empty(), "{:?}", outcome.limitations);
        let compose = outcome.plan().expect("a plan").to_compose().unwrap();
        assert!(compose.contains("stop_signal: SIGINT"), "{compose}");

        let hostile = definition.replace("SIGINT", "SIGINT\\n    privileged: true");
        let outcome = import("db", &hostile, None).unwrap();
        assert!(
            outcome.plan().is_none_or(|plan| plan.to_compose().is_err()),
            "a stop signal carried something else into the file"
        );
    }

    /// Plausible's TOTP key is 32 bytes of base64; an alphanumeric string of
    /// the same length decodes to 24 bytes and Plausible will not start.
    #[test]
    fn a_declared_secret_encoding_is_honoured() {
        let definition = r#"{"services": [{"name": "app", "image": "example/app:1.0", "isMain": true,
            "internalPort": 8000, "environment": [{"key": "A", "value": "${A}"}, {"key": "B", "value": "${B}"},
            {"key": "C", "value": "${C}"}]}]}"#;
        let config = r#"{"id": "app", "form_fields": [
            {"type": "random", "min": 32, "encoding": "base64", "env_variable": "A"},
            {"type": "random", "min": 64, "encoding": "hex", "env_variable": "B"},
            {"type": "random", "min": 32, "env_variable": "C"}]}"#;
        let outcome = import("app", definition, Some(config)).unwrap();
        let template = outcome.template.expect("importable");
        let format = |key: &str| {
            template
                .secrets
                .iter()
                .find(|s| s.key == key)
                .unwrap()
                .format
        };
        assert_eq!(format("A"), crate::setup::SecretFormat::Base64);
        assert_eq!(format("B"), crate::setup::SecretFormat::Hex);
        assert_eq!(format("C"), crate::setup::SecretFormat::Alphanumeric);
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
        // Waiting for a service to finish makes it a one-shot job.
        let job = import(
            "conditional",
            &definition.replace("service_healthy", "service_completed_successfully"),
            None,
        )
        .unwrap();
        assert!(job.plan().unwrap().is_job("db"));
        let blocked = import(
            "conditional",
            &definition.replace("service_healthy", "unknown"),
            None,
        )
        .unwrap();
        assert!(blocked.template.is_none());
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

    /// Sim's shape: a migration that must finish before the app starts, and
    /// a realtime server the browser reaches on a second address.
    const WITH_JOB_AND_COMPANION: &str = r#"{
      "services": [
        {"name": "app", "image": "example/app:1.0", "isMain": true, "internalPort": 3000,
         "environment": [
           {"key": "SOCKET_URL", "value": "${LOCAL_STORE_URL_REALTIME}"},
           {"key": "APP_URL", "value": "${LOCAL_STORE_URL}"}],
         "dependsOn": {"migrate": {"condition": "service_completed_successfully"},
                       "realtime": {"condition": "service_started"}}},
        {"name": "realtime", "image": "example/realtime:1.0",
         "addPorts": [{"hostPort": 3002, "containerPort": 3002}],
         "environment": [{"key": "PORT", "value": "${LOCAL_STORE_PORT_REALTIME}"}]},
        {"name": "migrate", "image": "example/migrate:1.0"}
      ]
    }"#;

    #[test]
    fn a_job_and_a_second_address_import_and_resolve_to_their_ports() {
        let outcome = import("app", WITH_JOB_AND_COMPANION, None).unwrap();
        assert!(outcome.is_importable(), "{:?}", outcome.limitations);
        let template = outcome.template.expect("a template");
        assert!(template.plan.is_job("migrate"));
        assert_eq!(
            template.plan.companions()[0].1,
            PublishedPort {
                host: 3002,
                container: 3002
            }
        );

        let mut filled = template.clone();
        filled.plan.set_companion_host("realtime", 43002);
        crate::setup::fill_platform_values(&mut filled.plan, 8080);
        let plan = filled.resolve(&BTreeMap::new(), &BTreeMap::new()).unwrap();
        let compose = plan.to_compose().unwrap();
        assert!(!compose.contains("${"), "{compose}");
        assert!(
            compose.contains("SOCKET_URL: \"http://localhost:43002\""),
            "{compose}"
        );
        assert!(compose.contains("PORT: \"43002\""), "{compose}");
        assert!(compose.contains("condition: service_completed_successfully"));
    }

    #[test]
    fn a_second_address_that_cannot_be_honoured_is_reported() {
        let refused = |ports: &str| {
            let definition = WITH_JOB_AND_COMPANION
                .replace(r#"[{"hostPort": 3002, "containerPort": 3002}]"#, ports);
            let outcome = import("app", &definition, None).unwrap();
            assert!(!outcome.is_importable(), "{ports} was accepted");
            assert!(
                outcome
                    .limitations
                    .iter()
                    .any(|limit| limit.feature() == "addPorts"),
                "{ports}: {:?}",
                outcome.limitations
            );
        };
        refused(r#"[{"hostPort": 53, "containerPort": 53, "udp": true}]"#);
        refused(r#"[{"hostPort": 53, "containerPort": 53, "tcp": false}]"#);
        refused(
            r#"[{"hostPort": 3002, "containerPort": 3002}, {"hostPort": 3003, "containerPort": 3003}]"#,
        );
        refused(r#"[{"hostPort": 3002, "containerPort": 3002, "interface": "0.0.0.0"}]"#);
        refused(r#"[{"containerPort": 3002}]"#);

        // A placeholder for a second address nobody publishes is never filled.
        let orphan = WITH_JOB_AND_COMPANION.replace("URL_REALTIME", "URL_OTHER");
        let outcome = import("app", &orphan, None).unwrap();
        assert!(outcome
            .limitations
            .iter()
            .any(|limit| limit.feature() == "platform placeholder"));
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
