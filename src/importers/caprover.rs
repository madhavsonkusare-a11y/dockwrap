//! Read-only importer for CapRover one-click app definitions.
//!
//! A CapRover app is a Compose fragment plus a `caproverOneClickApp` block of
//! typed variables, which is close to what a [`crate::setup::PlanTemplate`]
//! already is. This module maps one onto the other, or reports precisely what
//! it could not express.
//!
//! Upstream expressions are never evaluated. `$$cap_…` placeholders are
//! substituted from values this importer is willing to supply, and anything
//! left over is reported rather than guessed at. Nothing here installs
//! anything, and an import is never promoted to a reviewed recipe.
use super::{
    needs_input, not_modelled, plan_args, preferred_host_port, refused, ImportOutcome, Limitation,
};
use crate::plan::{DeploymentPlan, PlanArgs, PlanMount, PlanOverrides, PlanService, PublishedPort};
use crate::setup::{
    compile_setup_pattern, FieldKind, PlanTemplate, SecretFormat, SecretSpec, SetupField,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupInput {
    Field(SetupField),
    Secret(SecretSpec),
}

/// Import one normalized variable. Deployment substitution and duplicate-key
/// detection remain the responsibility of the future whole-definition adapter.
pub fn setup_variable(variable: &serde_json::Value) -> Result<SetupInput, String> {
    let object = variable
        .as_object()
        .ok_or("Expected setup variable object")?;
    if object.keys().any(|key| {
        !["id", "label", "description", "defaultValue", "validRegex"].contains(&key.as_str())
    }) {
        return Err("Unsupported setup variable property".into());
    }
    let id = object
        .get("id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.strip_prefix("$$cap_"))
        .filter(|s| {
            !s.is_empty()
                && s.len() <= 100
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        })
        .ok_or("Invalid CapRover variable identifier")?;
    if ["appname", "root_domain"].contains(&id) {
        return Err("Platform variable cannot be redeclared".into());
    }
    // Upstream writes both `$$cap_db_pass` and `$$cap_mariadb-db`. A hyphen
    // cannot appear in an environment key, so it folds to an underscore; the
    // caller detects the collision that folding can create.
    let key = format!("CAP_{}", normalized_key(id));
    let read = |name| -> Result<Option<&str>, String> {
        object
            .get(name)
            .map(|v| {
                v.as_str()
                    .ok_or_else(|| "Setup metadata must be text".into())
            })
            .transpose()
    };
    let label = read("label")?.unwrap_or(id);
    if label.len() > 256 {
        return Err("Setup label exceeds limit".into());
    }
    read("description")?;
    // Upstream writes ports and version numbers as YAML scalars, so a default
    // arrives as a number or a boolean as often as text. Both become the same
    // environment string in the end, so they are accepted rather than refused.
    //
    // A fractional number is not: YAML reads `13.10` as 13.1, and installing
    // that as an image tag would quietly fetch a different release.
    let default = match object.get("defaultValue") {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) => Some(text.clone()),
        Some(Value::Bool(flag)) => Some(flag.to_string()),
        Some(Value::Number(number)) if number.is_f64() => {
            return Err("Fractional default cannot be read back exactly".into())
        }
        Some(Value::Number(number)) => Some(number.to_string()),
        Some(_) => return Err("Setup metadata must be text".into()),
    };
    let default = default.as_deref();
    let rule = read("validRegex")?.filter(|s| !s.is_empty());
    if let Some(default) = default {
        if let Some(secret) = generated_secret(&key, default)? {
            // A rule attached to a generated credential was written for a
            // value a person types. It is honoured only when every value this
            // secret can produce is proven to satisfy it; an unproven pair is
            // refused rather than gambled on.
            if let Some(rule) = rule {
                let FieldKind::Pattern { pattern, .. } = validation_pattern(rule)? else {
                    return Err(
                        "Generated secret with validation rule needs compatibility review".into(),
                    );
                };
                if !secret.every_value_matches(&pattern) {
                    return Err(
                        "Generated secret is not proven to satisfy its validation rule".into(),
                    );
                }
            }
            return Ok(SetupInput::Secret(secret));
        }
    }
    let kind = rule
        .map(validation_pattern)
        .transpose()?
        .unwrap_or(FieldKind::Text {
            min_len: 0,
            max_len: 4096,
        });
    let required = if let FieldKind::Pattern { pattern, .. } = &kind {
        !compile_setup_pattern(pattern)?.is_match(b"")
    } else {
        false
    };
    let field = SetupField {
        key,
        label: label.into(),
        kind,
        required,
        default: default.map(str::to_owned),
        // Upstream has no reliable credential annotation. Keep answers masked
        // until reviewed metadata can explicitly mark a field non-sensitive.
        sensitive: true,
    };
    if let Some(default) = default {
        field
            .accept(Some(default))
            .map_err(|_| "Upstream default does not satisfy its setup rule")?;
    }
    Ok(SetupInput::Field(field))
}

/// The environment key a CapRover identifier folds to, ignoring the case and
/// separator differences that would otherwise produce two names for one key.
fn normalized_key(id: &str) -> String {
    id.to_ascii_uppercase().replace('-', "_")
}

/// Convert a deliberately narrow JavaScript regex literal to an ASCII setup
/// field. No flags, lookaround, backreferences, Unicode or escape extensions.
/// ASCII input and strict end anchors may reject inputs accepted upstream;
/// callers must surface that restriction rather than claim full JS parity.
pub fn validation_pattern(literal: &str) -> Result<FieldKind, String> {
    let unsupported = || "Unsupported CapRover validation pattern".to_string();
    if literal.len() > 2048 || !literal.is_ascii() {
        return Err(unsupported());
    }
    let body = literal
        .strip_prefix('/')
        .and_then(|s| s.strip_suffix('/'))
        .ok_or_else(unsupported)?;
    let mut converted = String::new();
    let mut chars = body.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '\\' => {
                let escaped = chars.next().ok_or_else(unsupported)?;
                if escaped == '/' {
                    converted.push('/');
                } else if "dDsSwW\\.^$|?*+()[]{}-".contains(escaped) {
                    converted.push('\\');
                    converted.push(escaped);
                } else {
                    return Err(unsupported());
                }
            }
            '/' => return Err(unsupported()),
            '(' if chars.peek() == Some(&'?') => return Err(unsupported()),
            c if c.is_ascii_control() => return Err(unsupported()),
            c => converted.push(c),
        }
    }
    compile_setup_pattern(&converted).map_err(|_| unsupported())?;
    Ok(FieldKind::Pattern {
        pattern: converted,
        min_len: 0,
        max_len: 4096,
    })
}

/// Recognize an entire generated-secret default, preserving output length.
/// Literal defaults return None; unknown/mixed expressions are refused.
/// Lengths below our secret policy are refused rather than silently enlarged.
pub fn generated_secret(key: &str, default: &str) -> Result<Option<SecretSpec>, String> {
    if !default.contains("$$") {
        return Ok(None);
    }
    let length = default
        .strip_prefix("$$cap_gen_random_hex(")
        .and_then(|value| value.strip_suffix(')'))
        .filter(|value| !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| (16..=256).contains(value))
        .ok_or("Unsupported generated-secret expression or length")?;
    if key.is_empty()
        || !key
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
        || key.as_bytes()[0].is_ascii_digit()
    {
        return Err("Invalid generated-secret placeholder name".into());
    }
    Ok(Some(SecretSpec {
        key: key.into(),
        length,
        format: SecretFormat::Hex,
    }))
}

// ---------------------------------------------------------------- adapter --

/// Root properties a version-4 definition may carry.
const ROOT_KEYS: &[&str] = &[
    "captainVersion",
    "services",
    "caproverOneClickApp",
    "volumes",
    "networks",
    "version",
];
/// Presentation metadata. None of it changes what gets deployed, so it is
/// allowed through and simply not used here.
const ONE_CLICK_KEYS: &[&str] = &[
    "instructions",
    "displayName",
    "description",
    "variables",
    "documentation",
    "isOfficial",
    "baseUrl",
];
/// Service properties this adapter understands and maps.
const SERVICE_KEYS: &[&str] = &[
    "healthcheck",
    "image",
    "command",
    "entrypoint",
    "hostname",
    "environment",
    "volumes",
    "depends_on",
    "caproverExtra",
    "restart",
    "ports",
    "expose",
    "container_name",
    "documentation",
];
/// `caproverExtra` properties this adapter understands.
const EXTRA_KEYS: &[&str] = &[
    "notExposeAsWebApp",
    "containerHttpPort",
    "dockerfileLines",
    "websocketSupport",
];
/// Service properties the plan model has no field for yet. Reported, never
/// dropped: silently ignoring a command or a health check changes how the
/// container starts.
const UNMODELLED_SERVICE: &[(&str, &str)] = &[
    ("user", "runs as a specific user"),
    ("networks", "declares custom networks"),
    ("logging", "configures a log driver"),
    ("env_file", "reads environment from a file"),
    ("working_dir", "sets a working directory"),
    ("labels", "sets container labels"),
    ("stdin_open", "keeps standard input open"),
    ("tty", "allocates a terminal"),
    ("volume", "uses a misspelt volumes key"),
];
/// Properties that would widen what a container may do.
const REFUSED_SERVICE: &[(&str, &str)] = &[
    ("cap_add", "adds Linux capabilities"),
    ("privileged", "runs privileged"),
    ("network_mode", "changes the network mode"),
    ("devices", "mounts host devices"),
    ("security_opt", "changes security options"),
    ("sysctls", "sets kernel parameters"),
    ("pid", "shares a process namespace"),
];

/// What the variables block produced, before any service is looked at.
#[derive(Default)]
struct Declared {
    fields: Vec<SetupField>,
    secrets: Vec<SecretSpec>,
    /// `$$cap_x` to the `${CAP_X}` placeholder that replaces it.
    placeholders: BTreeMap<String, String>,
    /// `$$cap_x` to the literal upstream default, used where a placeholder
    /// cannot survive — the image tag being the only such place.
    literals: BTreeMap<String, String>,
}

fn read_variables(
    definition: &serde_json::Map<String, Value>,
    limitations: &mut Vec<Limitation>,
) -> Declared {
    let mut declared = Declared::default();
    let mut seen_ids = BTreeSet::new();
    let one_click = definition
        .get("caproverOneClickApp")
        .and_then(Value::as_object);
    let Some(one_click) = one_click else {
        limitations.push(refused(
            "caproverOneClickApp",
            "the definition declares no one-click block",
        ));
        return declared;
    };
    for key in one_click.keys() {
        if !ONE_CLICK_KEYS.contains(&key.as_str()) {
            limitations.push(not_modelled(
                "one-click property",
                format!("declares unknown one-click property {key:?}"),
            ));
        }
    }
    for variable in one_click
        .get("variables")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let Some(id) = variable.get("id").and_then(Value::as_str) else {
            limitations.push(needs_input("setup variable", "a variable declares no id"));
            continue;
        };
        // Two variables that differ only in case, or only in whether they use
        // a hyphen or an underscore, become one environment key. Detecting
        // that here is what keeps one of them from silently overwriting the
        // other.
        if !seen_ids.insert(normalized_key(id)) {
            limitations.push(refused(
                "setup variable",
                format!("declares {id:?} more than once, or twice under one key"),
            ));
            continue;
        }
        match setup_variable(variable) {
            Ok(SetupInput::Field(field)) => {
                declared
                    .placeholders
                    .insert(id.to_owned(), format!("${{{}}}", field.key));
                if let Some(default) = &field.default {
                    declared.literals.insert(id.to_owned(), default.clone());
                }
                declared.fields.push(field);
            }
            Ok(SetupInput::Secret(secret)) => {
                declared
                    .placeholders
                    .insert(id.to_owned(), format!("${{{}}}", secret.key));
                declared.secrets.push(secret);
            }
            // The primitive's message names the reason without echoing values.
            Err(reason) => {
                let default = variable.get("defaultValue").and_then(Value::as_str);
                if default.is_some_and(|value| value.contains(PUBLIC_DOMAIN)) {
                    limitations.push(refused(
                        "public domain",
                        "a setup value is built from the platform's own domain",
                    ));
                } else {
                    limitations.push(needs_input("setup variable", reason));
                }
            }
        }
    }
    declared
}

/// The domain CapRover routes every app under.
///
/// An app that builds *its own* address out of this is asking for something
/// this project does have: the loopback address it is about to be published
/// on. That is filled in as a placeholder the installer resolves once the port
/// is settled.
///
/// An app that builds a *sibling's* public address out of it is asking for a
/// second routable endpoint, which the plan model does not have and will not
/// invent. Those are refused, because substituting loopback there would point
/// the app at itself.
const PUBLIC_DOMAIN: &str = "$$cap_root_domain";

/// The app's own address on the platform, as opposed to a sibling's.
const OWN_HOST: &str = "$$cap_appname.$$cap_root_domain";

/// Rewrite a value that is *entirely* the app's own address into the
/// placeholder the installer fills.
///
/// Deliberately all-or-nothing. A value that merely contains the address —
/// `wss://…/cable`, a comma-separated host list, a URL with a path, a
/// sibling's `$$cap_appname-web.$$cap_root_domain` — is left alone and refused
/// downstream. Rewriting those would change a scheme, point an app at itself,
/// or hand it half an address, and being wrong there is worse than importing
/// fewer apps.
fn rewrite_own_address(text: &str) -> String {
    let trimmed = text.strip_suffix('/').unwrap_or(text);
    let slash = if text.ends_with('/') { "/" } else { "" };
    for scheme in ["https://", "http://"] {
        if trimmed == format!("{scheme}{OWN_HOST}") {
            return format!("${{{}}}{slash}", crate::setup::PLATFORM_URL);
        }
    }
    if text == OWN_HOST {
        return format!("${{{}}}", crate::setup::PLATFORM_HOST);
    }
    text.to_owned()
}

/// Why a value could not be filled in. Both are refusals to guess; they are
/// separated because one of them is a product boundary worth counting on its
/// own, and the other is an unknown to be looked at case by case.
enum Unfilled {
    /// Needs the public hostname this platform routes through.
    PublicDomain,
    /// Some other `$$cap_…` expression. Never evaluated, never guessed at.
    Unknown,
}

/// Substitute the platform's own expressions, longest identifier first so a
/// shorter name can never eat the prefix of a longer one.
///
/// `values` supplies a replacement for every `$$cap_…` this adapter is willing
/// to resolve. Anything left carrying `$$` afterwards is reported, never
/// evaluated and never approximated.
fn substitute(text: &str, values: &BTreeMap<String, String>) -> Result<String, Unfilled> {
    if text.contains(PUBLIC_DOMAIN) {
        return Err(Unfilled::PublicDomain);
    }
    let mut keys: Vec<&String> = values.keys().collect();
    keys.sort_by_key(|key| std::cmp::Reverse(key.len()));
    let mut out = text.to_owned();
    for key in keys {
        if out.contains(key.as_str()) {
            out = out.replace(key.as_str(), &values[key]);
        }
    }
    if out.contains("$$") {
        return Err(Unfilled::Unknown);
    }
    Ok(out)
}

/// Turn an unfilled value into the limitation that explains it, without ever
/// echoing the value itself: it may be half-substituted and carrying a secret.
fn unfilled(reason: &Unfilled, feature: &str, subject: &str) -> Limitation {
    match reason {
        Unfilled::PublicDomain => refused(
            "public domain",
            format!("{subject} builds a public address from the platform's own domain"),
        ),
        Unfilled::Unknown => needs_input(
            feature,
            format!("{subject} uses a platform expression this importer will not guess at"),
        ),
    }
}

/// Map one CapRover definition onto a plan template, or explain why not.
///
/// `definition` is the app's YAML normalized to JSON; this function never
/// reads the archive, never resolves a network reference and never evaluates
/// an upstream expression.
pub fn import(id: &str, definition: &str) -> Result<ImportOutcome, String> {
    let root: Value = serde_json::from_str(definition)
        .map_err(|error| format!("{id}: definition is not readable JSON: {error}"))?;
    let Some(root) = root.as_object() else {
        return Err(format!("{id}: definition is not an object"));
    };
    let mut limitations: Vec<Limitation> = Vec::new();

    // Only version 4 is understood. An unknown version may mean anything.
    let version = root.get("captainVersion");
    let version_is_four = version.and_then(Value::as_u64) == Some(4)
        || version.and_then(Value::as_str) == Some("4")
        || version.and_then(Value::as_f64) == Some(4.0);
    if !version_is_four {
        limitations.push(refused(
            "captainVersion",
            "declares a captainVersion this importer does not understand",
        ));
    }
    for key in root.keys() {
        if !ROOT_KEYS.contains(&key.as_str()) {
            limitations.push(not_modelled(
                "root property",
                format!("declares unknown root property {key:?}"),
            ));
        }
    }
    if root.contains_key("networks") {
        limitations.push(refused(
            "networks",
            "declares custom networks, which the managed project does not model",
        ));
    }

    let declared = read_variables(root, &mut limitations);

    let services = root.get("services").and_then(Value::as_object);
    let Some(services) = services else {
        return Err(format!("{id}: definition declares no services"));
    };

    // Names first: environment values reference sibling services by their
    // platform name, so the mapping has to exist before anything is filled in.
    let mut names: BTreeMap<String, String> = BTreeMap::new();
    for name in services.keys() {
        let mapped = match name.as_str() {
            "$$cap_appname" => id.to_owned(),
            other => match other.strip_prefix("$$cap_appname-") {
                Some(suffix) => suffix.to_owned(),
                None => {
                    limitations.push(refused(
                        "service name",
                        "names a service outside this app's own namespace",
                    ));
                    continue;
                }
            },
        };
        names.insert(name.clone(), mapped);
    }

    // One substitution table for everything a value may mention.
    let mut values = declared.placeholders.clone();
    for (platform, mapped) in &names {
        // `srv-captain--<app>` is how one CapRover service addresses another;
        // inside one Compose project that is just the service name.
        values.insert(format!("srv-captain--{platform}"), mapped.clone());
    }
    values.insert("$$cap_appname".into(), id.to_owned());
    // The image tag is rendered before any answer exists, so a placeholder
    // there would reach Compose unresolved. Pin it from the upstream default
    // instead, and do not offer a field that could not change anything.
    let mut image_values = declared.literals.clone();
    image_values.insert("$$cap_appname".into(), id.to_owned());

    let mut plan_services = Vec::new();
    let mut named_volumes = Vec::new();
    let mut endpoints: Vec<(String, u16)> = Vec::new();
    // A service this adapter could not map takes its endpoint with it. Saying
    // the app has no endpoint on top of that would report one problem twice
    // and hide which one actually needs fixing.
    let mut dropped_services = 0_usize;

    for (platform_name, service) in services {
        let Some(service) = service.as_object() else {
            limitations.push(refused(
                "service",
                "declares a service that is not an object",
            ));
            dropped_services += 1;
            continue;
        };
        let Some(name) = names.get(platform_name).cloned() else {
            dropped_services += 1;
            continue;
        };

        for key in service.keys() {
            let key = key.as_str();
            if SERVICE_KEYS.contains(&key) {
                continue;
            }
            if let Some((_, detail)) = REFUSED_SERVICE.iter().find(|(k, _)| *k == key) {
                limitations.push(refused(key, format!("{name} {detail}")));
            } else if let Some((_, detail)) = UNMODELLED_SERVICE.iter().find(|(k, _)| *k == key) {
                limitations.push(not_modelled(key, format!("{name} {detail}")));
            } else {
                limitations.push(not_modelled(
                    "service property",
                    format!("{name} declares unknown property {key:?}"),
                ));
            }
        }

        let extra = service
            .get("caproverExtra")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        for key in extra.keys() {
            if !EXTRA_KEYS.contains(&key.as_str()) {
                limitations.push(not_modelled(
                    "caproverExtra property",
                    format!("{name} declares unknown platform property {key:?}"),
                ));
            }
        }
        if extra.contains_key("dockerfileLines") {
            // The platform builds an image from these lines. We install
            // published images and never build one.
            limitations.push(refused(
                "dockerfileLines",
                format!("{name} is built from a Dockerfile rather than a published image"),
            ));
        }

        let image = match service.get("image").and_then(Value::as_str) {
            Some(image) => match substitute(image, &image_values) {
                Ok(image) => image,
                Err(reason) => {
                    limitations.push(unfilled(&reason, "image", &name));
                    dropped_services += 1;
                    continue;
                }
            },
            None => {
                limitations.push(refused(
                    "image",
                    format!("{name} declares no image to install"),
                ));
                dropped_services += 1;
                continue;
            }
        };

        let mut environment = Vec::new();
        match service.get("environment") {
            None => {}
            Some(Value::Object(map)) => {
                for (key, value) in map {
                    let text = match value {
                        Value::String(text) => text.clone(),
                        Value::Number(number) => number.to_string(),
                        Value::Bool(flag) => flag.to_string(),
                        Value::Null => String::new(),
                        _ => {
                            limitations.push(not_modelled(
                                "environment value",
                                format!("{name} sets {key} to a structured value"),
                            ));
                            continue;
                        }
                    };
                    // Only an environment value may carry the app's own
                    // address: a hostname or an image tag has no use for a URL.
                    match substitute(&rewrite_own_address(&text), &values) {
                        Ok(filled) => environment.push((key.clone(), filled)),
                        Err(reason) => limitations.push(unfilled(&reason, "environment", &name)),
                    }
                }
            }
            Some(_) => limitations.push(not_modelled(
                "environment",
                format!("{name} declares environment in a form this importer does not read"),
            )),
        }

        let mut mounts = Vec::new();
        for mount in service
            .get("volumes")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default()
        {
            let Some(text) = mount.as_str() else {
                limitations.push(not_modelled(
                    "volume",
                    format!("{name} declares a mount in long syntax"),
                ));
                continue;
            };
            let Some((source, target)) = text.split_once(':') else {
                limitations.push(refused("volume", format!("{name} declares a bare mount")));
                continue;
            };
            let (target, read_only) = match target.strip_suffix(":ro") {
                Some(path) => (path, true),
                None => (target, false),
            };
            if source.starts_with('/') || source.starts_with('.') {
                limitations.push(refused(
                    "host path",
                    format!("{name} mounts a path from this computer"),
                ));
                continue;
            }
            let volume = match substitute(source, &values) {
                Ok(volume) => volume,
                Err(reason) => {
                    limitations.push(unfilled(&reason, "volume", &name));
                    continue;
                }
            };
            if !named_volumes.contains(&volume) {
                named_volumes.push(volume.clone());
            }
            mounts.push(PlanMount::Volume {
                name: volume,
                target: target.to_owned(),
                read_only,
            });
        }

        let (dependencies, health_names) = super::dependencies(service.get("depends_on"))
            .unwrap_or_else(|reason| {
                limitations.push(not_modelled("depends_on", format!("{name}: {reason}")));
                Default::default()
            });
        let mut depends_on = Vec::new();
        let mut healthy_dependencies = BTreeSet::new();
        for dependency in dependencies {
            match names.get(&dependency) {
                Some(mapped) => {
                    depends_on.push(mapped.clone());
                    if health_names.contains(&dependency) {
                        healthy_dependencies.insert(mapped.clone());
                    }
                }
                None => limitations.push(refused(
                    "depends_on",
                    format!("{name} waits on a service this definition does not declare"),
                )),
            }
        }

        // Startup overrides are carried through rather than dropped: a
        // definition that replaces the image's command means it.
        let mut overrides = PlanOverrides {
            healthy_dependencies,
            ..Default::default()
        };
        if let Some(value) = service.get("healthcheck") {
            match crate::plan::PlanHealthcheck::from_compose(value) {
                Ok(check) => overrides.healthcheck = Some(check),
                Err(reason) => {
                    limitations.push(not_modelled("healthcheck", format!("{name}: {reason}")))
                }
            }
        }
        for (field, value) in [
            ("command", service.get("command")),
            ("entrypoint", service.get("entrypoint")),
        ] {
            let Some(value) = value else { continue };
            let filled = plan_args(value).and_then(|args| match args {
                PlanArgs::Shell(line) => substitute(&line, &values).ok().map(PlanArgs::Shell),
                PlanArgs::Exec(parts) => parts
                    .iter()
                    .map(|part| substitute(part, &values).ok())
                    .collect::<Option<Vec<String>>>()
                    .map(PlanArgs::Exec),
            });
            match filled {
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
                // Nearly every hostname upstream is the app's public address,
                // so the reason has to be the public domain rather than a
                // generic unreadable-hostname note.
                Some(text) => match substitute(text, &values) {
                    Ok(hostname) => overrides.hostname = Some(hostname),
                    Err(reason) => limitations.push(unfilled(&reason, "hostname", &name)),
                },
                None => limitations.push(not_modelled(
                    "hostname",
                    format!("{name} declares a hostname this importer cannot read"),
                )),
            }
        }

        // The platform publishes one HTTP port per web-facing app; a database
        // carries notExposeAsWebApp and stays on the internal network.
        // Upstream writes this both as a YAML boolean and as the quoted string
        // "true". Reading only one of the two would quietly publish a database
        // to the host, so an unrecognised value is refused rather than assumed.
        let hidden = match extra.get("notExposeAsWebApp") {
            None => Some(false),
            Some(Value::Bool(flag)) => Some(*flag),
            Some(Value::String(text)) => match text.trim().to_ascii_lowercase().as_str() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            },
            Some(_) => None,
        };
        let Some(hidden) = hidden else {
            limitations.push(refused(
                "notExposeAsWebApp",
                format!("{name} declares web exposure in a form this importer cannot read"),
            ));
            plan_services.push(PlanService {
                name,
                image,
                environment,
                published: None,
                mounts,
                depends_on,
                overrides,
            });
            continue;
        };
        let exposed = !hidden;
        if exposed {
            let port = match extra.get("containerHttpPort") {
                None => Some(80),
                Some(Value::Number(number)) => number.as_u64().and_then(|p| u16::try_from(p).ok()),
                Some(Value::String(text)) => text.parse::<u16>().ok(),
                Some(_) => None,
            };
            match port.filter(|port| *port > 0) {
                Some(port) => endpoints.push((name.clone(), port)),
                None => limitations.push(refused(
                    "containerHttpPort",
                    format!("{name} declares a port this importer cannot read"),
                )),
            }
        }
        if service.contains_key("ports") || service.contains_key("expose") {
            limitations.push(refused(
                "ports",
                format!("{name} publishes ports of its own beside the platform endpoint"),
            ));
        }

        plan_services.push(PlanService {
            name,
            image,
            environment,
            published: None,
            mounts,
            depends_on,
            overrides,
        });
    }

    // Exactly one address reaches this computer. Choosing between two would be
    // a product decision, not an import one.
    match endpoints.len() {
        1 => {
            let (service, container) = &endpoints[0];
            if let Some(target) = plan_services.iter_mut().find(|s| &s.name == service) {
                target.published = Some(PublishedPort {
                    host: preferred_host_port(*container),
                    container: *container,
                });
            }
        }
        0 if dropped_services == 0 => limitations.push(refused(
            "endpoint",
            "declares no web endpoint, so there would be nothing to open",
        )),
        0 => {}
        count => limitations.push(refused(
            "endpoint",
            format!("declares {count} web endpoints and the plan model publishes one"),
        )),
    }

    if !limitations.is_empty() {
        return Ok(ImportOutcome {
            id: id.to_owned(),
            template: None,
            limitations,
        });
    }

    named_volumes.sort();
    let plan = DeploymentPlan {
        id: id.to_owned(),
        services: plan_services,
        named_volumes,
    };

    // Only what something actually reads survives. A version variable spent
    // pinning the image reaches no environment value, and offering it as a
    // field would invite someone to change a number that changes nothing.
    let referenced = |key: &str| {
        let placeholder = format!("${{{key}}}");
        plan.services.iter().any(|service| {
            service
                .environment
                .iter()
                .any(|(_, value)| value.contains(&placeholder))
        })
    };
    let fields: Vec<SetupField> = declared
        .fields
        .into_iter()
        .filter(|field| referenced(&field.key))
        .collect();
    let secrets: Vec<SecretSpec> = declared
        .secrets
        .into_iter()
        .filter(|secret| referenced(&secret.key))
        .collect();

    let template = PlanTemplate {
        plan,
        fields,
        secrets,
    };
    // The shared rules have the final say, and a rule this adapter failed to
    // anticipate must surface as a limitation rather than a broken plan.
    if let Err(reason) = template.validate() {
        limitations.push(refused("plan rule", reason));
    } else if let Err(reason) = template.plan.to_compose() {
        limitations.push(refused("plan rule", reason));
    }
    Ok(ImportOutcome {
        id: id.to_owned(),
        template: limitations.is_empty().then_some(template),
        limitations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A single web service with a named volume and a fixed version default:
    /// the simplest shape the checklist asks to prove first.
    const SIMPLE: &str = r#"{
      "captainVersion": 4,
      "services": {
        "$$cap_appname": {
          "image": "actualbudget/actual-server:$$cap_version",
          "volumes": ["$$cap_appname-data:/data"],
          "restart": "unless-stopped",
          "caproverExtra": {"containerHttpPort": "5006"}
        }
      },
      "caproverOneClickApp": {
        "displayName": "Actual",
        "description": "Budgeting.",
        "instructions": {"start": "hello"},
        "variables": [
          {"id": "$$cap_version", "label": "Version", "defaultValue": "23.8.1-alpine"}
        ]
      }
    }"#;

    #[test]
    fn a_single_web_service_maps_to_a_plan_with_its_volume_and_endpoint() {
        let outcome = import("actual", SIMPLE).unwrap();
        assert!(outcome.limitations.is_empty(), "{:?}", outcome.limitations);
        let template = outcome.template.as_ref().expect("a template");
        let plan = &template.plan;
        assert_eq!(plan.services.len(), 1);
        // The version variable is spent pinning the image, so it must not also
        // appear as a field that could not change anything.
        assert_eq!(
            plan.services[0].image,
            "actualbudget/actual-server:23.8.1-alpine"
        );
        assert!(template.fields.is_empty(), "{:?}", template.fields);
        let (service, port) = plan.published().expect("an endpoint");
        assert_eq!(service.name, "actual");
        assert_eq!(port.container, 5006);
        assert_eq!(port.host, 5006);
        assert_eq!(plan.named_volumes, vec!["actual-data"]);
        let compose = plan.to_compose().expect("plan should render");
        assert!(compose.contains("actual-data:/data"), "{compose}");
        assert!(
            !compose.contains("$$"),
            "a platform expression reached Compose"
        );
    }

    /// Web plus database: service DNS substitution, a generated credential
    /// shared by both services, an unpublished database and dependency order.
    const PAIR: &str = r#"{
      "captainVersion": 4,
      "services": {
        "$$cap_appname": {
          "image": "example/web:1.4.0",
          "environment": {
            "DATABASE_URL": "postgres://app:$$cap_pg_password@srv-captain--$$cap_appname-db:5432/app",
            "SITE_NAME": "$$cap_site"
          },
          "depends_on": ["$$cap_appname-db"],
          "caproverExtra": {"containerHttpPort": 3000}
        },
        "$$cap_appname-db": {
          "image": "postgres:16.10-alpine",
          "environment": {"POSTGRES_PASSWORD": "$$cap_pg_password"},
          "volumes": ["$$cap_appname-db-data:/var/lib/postgresql/data"],
          "caproverExtra": {"notExposeAsWebApp": true}
        }
      },
      "caproverOneClickApp": {
        "displayName": "Example",
        "description": "Example.",
        "instructions": {"start": "hello"},
        "variables": [
          {"id": "$$cap_pg_password", "defaultValue": "$$cap_gen_random_hex(32)"},
          {"id": "$$cap_site", "label": "Site name", "defaultValue": "Notes"}
        ]
      }
    }"#;

    #[test]
    fn a_web_app_and_its_database_share_a_credential_and_keep_the_database_private() {
        let outcome = import("example", PAIR).unwrap();
        assert!(outcome.limitations.is_empty(), "{:?}", outcome.limitations);
        let template = outcome.template.as_ref().expect("a template");
        let plan = &template.plan;

        // One generated secret, referenced by both services under one key.
        assert_eq!(template.secrets.len(), 1);
        let secret = &template.secrets[0].key;
        let web = plan.services.iter().find(|s| s.name == "example").unwrap();
        let db = plan.services.iter().find(|s| s.name == "db").unwrap();
        let url = &web
            .environment
            .iter()
            .find(|(key, _)| key == "DATABASE_URL")
            .unwrap()
            .1;
        // The platform's own service address becomes the Compose service name.
        assert!(url.contains("@db:5432"), "{url}");
        assert!(url.contains(&format!("${{{secret}}}")), "{url}");
        assert_eq!(
            db.environment
                .iter()
                .find(|(key, _)| key == "POSTGRES_PASSWORD")
                .unwrap()
                .1,
            format!("${{{secret}}}")
        );

        assert_eq!(web.depends_on, vec!["db".to_owned()]);
        assert!(db.published.is_none(), "the database reached the host");
        assert_eq!(plan.published().unwrap().0.name, "example");
        assert_eq!(template.fields.len(), 1);
        assert_eq!(template.fields[0].key, "CAP_SITE");

        // It has to resolve and render, not merely construct.
        let answers = [("CAP_SITE".to_owned(), "Notes".to_owned())]
            .into_iter()
            .collect();
        let secrets = [(secret.clone(), "a".repeat(32))].into_iter().collect();
        let compose = template
            .resolve(&answers, &secrets)
            .expect("template should resolve")
            .to_compose()
            .expect("plan should render");
        assert!(!compose.contains("${"), "a placeholder reached Compose");
        assert!(!compose.contains("srv-captain"), "{compose}");
    }

    fn only_limitation(definition: &str) -> Vec<String> {
        let outcome = import("app", definition).unwrap();
        assert!(!outcome.is_importable(), "definition should not import");
        outcome
            .limitations
            .iter()
            .map(|limit| limit.feature().to_owned())
            .collect()
    }

    #[test]
    fn definitions_that_cannot_be_expressed_are_named_rather_than_trimmed() {
        // A built image, not a published one.
        let built = SIMPLE.replace(
            r#""containerHttpPort": "5006""#,
            r#""containerHttpPort": "5006", "dockerfileLines": ["FROM x"]"#,
        );
        assert!(only_limitation(&built).contains(&"dockerfileLines".to_owned()));

        // A host path, and a Docker socket in particular.
        let host = SIMPLE.replace(
            r#""$$cap_appname-data:/data""#,
            r#""/var/run/docker.sock:/var/run/docker.sock""#,
        );
        assert!(only_limitation(&host).contains(&"host path".to_owned()));

        // A capability the reviewed recipes refuse.
        let privileged = SIMPLE.replace(
            r#""restart": "unless-stopped""#,
            r#""cap_add": ["NET_ADMIN"]"#,
        );
        assert!(only_limitation(&privileged).contains(&"cap_add".to_owned()));

        // A property that is still not modelled must still be named.
        let user = SIMPLE.replace(r#""restart": "unless-stopped""#, r#""user": "1000:1000""#);
        assert!(only_limitation(&user).contains(&"user".to_owned()));

        // An expression this importer will not guess at, in an image tag.
        // Only the image reference changes; the variable stays declared, so
        // what is refused is the undeclared name rather than a missing block.
        let unknown = SIMPLE.replace(
            "actual-server:$$cap_version",
            "actual-server:$$cap_undeclared_thing",
        );
        let features = only_limitation(&unknown);
        assert!(features.contains(&"image".to_owned()), "{features:?}");

        // A version this importer does not understand.
        let version = SIMPLE.replace(r#""captainVersion": 4"#, r#""captainVersion": 5"#);
        assert!(only_limitation(&version).contains(&"captainVersion".to_owned()));

        // Two variables that would collide once both became one key.
        let collision = SIMPLE.replace(
            r#"{"id": "$$cap_version", "label": "Version", "defaultValue": "23.8.1-alpine"}"#,
            r#"{"id": "$$cap_version", "defaultValue": "1"}, {"id": "$$cap_VERSION", "defaultValue": "2"}"#,
        );
        assert!(only_limitation(&collision).contains(&"setup variable".to_owned()));

        // Nothing to open.
        let headless = SIMPLE.replace(
            r#""containerHttpPort": "5006""#,
            r#""notExposeAsWebApp": true"#,
        );
        assert!(only_limitation(&headless).contains(&"endpoint".to_owned()));
    }

    #[test]
    fn a_refused_definition_never_echoes_a_value_it_was_given() {
        let secret = SIMPLE.replace(
            r#""defaultValue": "23.8.1-alpine""#,
            r#""defaultValue": "hunter2-should-not-appear", "validRegex": "/^(?=x)/""#,
        );
        let outcome = import("app", &secret).unwrap();
        for limitation in &outcome.limitations {
            assert!(
                !limitation.detail().contains("hunter2"),
                "{}",
                limitation.detail()
            );
        }
    }

    #[test]
    fn a_command_is_carried_through_in_the_form_upstream_wrote_it() {
        // A bare string is split the way a shell would split it; a list is
        // not. Rendering one as the other would change what the container runs.
        let shell = SIMPLE.replace(
            r#""restart": "unless-stopped""#,
            r#""command": "serve --fast", "entrypoint": ["/bin/sh", "-c"], "hostname": "notes""#,
        );
        let outcome = import("actual", &shell).unwrap();
        assert!(outcome.limitations.is_empty(), "{:?}", outcome.limitations);
        let compose = outcome.plan().unwrap().to_compose().unwrap();
        // Quoted by the same rule every other scalar goes through; still one
        // string, still split by Compose the way a shell would split it.
        assert!(
            compose.contains("    command: \"serve --fast\"\n"),
            "{compose}"
        );
        assert!(
            compose.contains(
                "    entrypoint:
      - /bin/sh
      - \"-c\"
"
            ),
            "{compose}"
        );
        assert!(
            compose.contains(
                "    hostname: notes
"
            ),
            "{compose}"
        );

        let listed = SIMPLE.replace(
            r#""restart": "unless-stopped""#,
            r#""command": ["serve", "--fast"]"#,
        );
        let compose = import("actual", &listed)
            .unwrap()
            .plan()
            .unwrap()
            .to_compose()
            .unwrap();
        assert!(
            compose.contains(
                "    command:
      - serve
"
            ),
            "{compose}"
        );
    }

    #[test]
    fn healthcheck_is_preserved_or_reported_never_dropped() {
        let mut definition: serde_json::Value = serde_json::from_str(SIMPLE).unwrap();
        definition["services"]["$$cap_appname"]["healthcheck"] = serde_json::json!({"test":["CMD","curl","-f","http://localhost:5006"], "interval":"10s"});
        let outcome = import("actual", &definition.to_string()).unwrap();
        assert!(outcome.limitations.is_empty(), "{:?}", outcome.limitations);
        assert!(outcome.plan().unwrap().services[0]
            .overrides
            .healthcheck
            .is_some());
        definition["services"]["$$cap_appname"]["healthcheck"]["test"] =
            serde_json::json!("echo $$cap_password");
        let outcome = import("actual", &definition.to_string()).unwrap();
        assert!(outcome.template.is_none());
        assert!(outcome
            .limitations
            .iter()
            .any(|l| l.feature() == "healthcheck"));
    }

    #[test]
    fn a_hyphenated_identifier_folds_to_one_key_and_a_fold_collision_is_caught() {
        let value = serde_json::json!({"id":"$$cap_mariadb-db", "defaultValue":"app"});
        let SetupInput::Field(field) = setup_variable(&value).unwrap() else {
            panic!("expected field")
        };
        assert_eq!(field.key, "CAP_MARIADB_DB");

        // `$$cap_a-b` and `$$cap_a_b` fold to the same environment key. Letting
        // both through would have one silently overwrite the other.
        let colliding = SIMPLE.replace(
            r#"{"id": "$$cap_version", "label": "Version", "defaultValue": "23.8.1-alpine"}"#,
            r#"{"id": "$$cap_a-b", "defaultValue": "1"}, {"id": "$$cap_a_b", "defaultValue": "2"}"#,
        );
        let outcome = import("app", &colliding).unwrap();
        assert!(outcome
            .limitations
            .iter()
            .any(|limit| limit.feature() == "setup variable"));
    }

    #[test]
    fn web_exposure_is_read_whether_it_is_a_boolean_or_the_quoted_string() {
        // Upstream writes both. Reading only the boolean form left every
        // database looking like a second web endpoint, and an app whose
        // endpoint rule let that through would have published it to the host.
        for written in ["true", "\"true\""] {
            let definition = PAIR.replace(
                "\"notExposeAsWebApp\": true",
                &format!("\"notExposeAsWebApp\": {written}"),
            );
            let outcome = import("example", &definition).unwrap();
            assert!(
                outcome.limitations.is_empty(),
                "{written}: {:?}",
                outcome.limitations
            );
            let plan = outcome.plan().expect("a plan");
            assert_eq!(plan.published().expect("one endpoint").0.name, "example");
            assert!(
                plan.services
                    .iter()
                    .find(|s| s.name == "db")
                    .unwrap()
                    .published
                    .is_none(),
                "{written}: the database reached the host"
            );
        }
        // Anything else is refused rather than assumed either way.
        let unreadable = PAIR.replace(
            "\"notExposeAsWebApp\": true",
            "\"notExposeAsWebApp\": \"yes\"",
        );
        let outcome = import("example", &unreadable).unwrap();
        assert!(outcome
            .limitations
            .iter()
            .any(|limit| limit.feature() == "notExposeAsWebApp"));
    }

    #[test]
    fn a_privileged_container_port_is_published_above_the_reserved_range() {
        let privileged = SIMPLE.replace(
            r#""containerHttpPort": "5006""#,
            r#""containerHttpPort": 80"#,
        );
        let outcome = import("app", &privileged).unwrap();
        assert!(outcome.limitations.is_empty(), "{:?}", outcome.limitations);
        let (_, port) = outcome.plan().unwrap().published().unwrap();
        assert_eq!((port.container, port.host), (80, 8080));
    }

    #[test]
    fn an_absent_container_port_uses_the_platform_default_of_eighty() {
        let implicit = SIMPLE.replace(
            "\"restart\": \"unless-stopped\",
          \"caproverExtra\": {\"containerHttpPort\": \"5006\"}",
            "\"restart\": \"unless-stopped\"",
        );
        assert!(
            !implicit.contains("containerHttpPort"),
            "fixture edit missed"
        );
        let outcome = import("app", &implicit).unwrap();
        assert!(outcome.limitations.is_empty(), "{:?}", outcome.limitations);
        let (_, port) = outcome.plan().unwrap().published().unwrap();
        assert_eq!((port.container, port.host), (80, 8080));
    }

    #[test]
    fn variables_keep_defaults_validation_and_generated_secret_identity() {
        let value = serde_json::json!({"id":"$$cap_user", "label":"User", "defaultValue":"admin", "validRegex":"/^[a-z]+$/"});
        let SetupInput::Field(field) = setup_variable(&value).unwrap() else {
            panic!("expected field")
        };
        assert!(field.required && field.sensitive);
        assert_eq!(field.key, "CAP_USER");
        assert_eq!(field.accept(None).unwrap(), "admin");
        assert!(field.accept(Some("INVALID")).is_err());
        let secret =
            serde_json::json!({"id":"$$cap_password", "defaultValue":"$$cap_gen_random_hex(32)"});
        assert!(matches!(
            setup_variable(&secret).unwrap(),
            SetupInput::Secret(SecretSpec {
                length: 32,
                format: SecretFormat::Hex,
                ..
            })
        ));
    }

    #[test]
    fn variable_import_never_drops_rules_or_evaluates_platform_defaults() {
        for value in [
            serde_json::json!({"id":"$$cap_x", "defaultValue":"private", "validRegex":"/^numbers$/"}),
            serde_json::json!({"id":"$$cap_x", "defaultValue":"$$cap_root_domain"}),
            // Every generated character must satisfy the rule; a hex value can
            // be all letters, so an all-digits rule is not provable.
            serde_json::json!({"id":"$$cap_x", "defaultValue":"$$cap_gen_random_hex(32)", "validRegex":r"/^\d+$/"}),
            // Anchored at both ends and too short for a 32-character value.
            serde_json::json!({"id":"$$cap_x", "defaultValue":"$$cap_gen_random_hex(32)", "validRegex":"/^.{1,8}$/"}),
            // A lookahead the pattern translator refuses outright.
            serde_json::json!({"id":"$$cap_x", "defaultValue":"$$cap_gen_random_hex(32)", "validRegex":"/^(?=.*x).+$/"}),
            serde_json::json!({"id":"$$cap_x", "unknown":true}),
            serde_json::json!({"id":"$$cap_appname"}),
        ] {
            let error = setup_variable(&value).unwrap_err();
            assert!(!error.contains("private"));
        }
    }

    #[test]
    fn common_ascii_rules_translate_without_evaluating_javascript() {
        for literal in [
            r"/.{1,}/",
            r"/^(true|false)$/",
            r"/^([a-zA-Z0-9])+$/",
            r"/^\d+$/",
            r"/^([^\s^\/])+$/",
        ] {
            let FieldKind::Pattern { pattern, .. } = validation_pattern(literal).unwrap() else {
                panic!("expected pattern")
            };
            let sample: &[u8] = if literal.contains("true") {
                b"true"
            } else {
                b"123"
            };
            assert!(compile_setup_pattern(&pattern).unwrap().is_match(sample));
        }
    }

    #[test]
    fn unsupported_regex_semantics_are_named_limitations() {
        for literal in [
            r"/x/i",
            r"/x/g",
            r"/x/u",
            r"/x/m",
            r"/(?=x)x/",
            r"/(x)\1/",
            r"/\p{L}/",
            r"/\u0061/",
            r"/[a-z/",
            "missing-delimiters",
            "/é/",
            r"/a/b/",
        ] {
            assert_eq!(
                validation_pattern(literal).unwrap_err(),
                "Unsupported CapRover validation pattern"
            );
        }
    }

    #[test]
    fn hex_declarations_preserve_character_length() {
        for length in [16, 32, 64, 256] {
            let secret =
                generated_secret("CAP_PASSWORD", &format!("$$cap_gen_random_hex({length})"))
                    .unwrap()
                    .unwrap();
            assert_eq!(secret.length, length);
            assert_eq!(secret.format, SecretFormat::Hex);
            assert_eq!(secret.generate().unwrap().len(), length);
        }
        assert_eq!(generated_secret("CAP_PASSWORD", "literal").unwrap(), None);
    }

    #[test]
    fn refuses_mixed_unknown_and_out_of_policy_expressions_without_echoing() {
        for value in [
            "$$cap_gen_random_hex(8)",
            "$$cap_gen_random_hex(257)",
            "$$cap_gen_random_hex(-1)",
            "$$cap_gen_random_hex(32)private",
            "private$$cap_gen_random_hex(32)",
            "$$cap_unknown(32)",
            "$$cap_gen_random_hex(99999999999999999999999999999)",
        ] {
            let error = generated_secret("CAP_PASSWORD", value).unwrap_err();
            assert!(!error.contains(value));
            assert!(!error.contains("private"));
        }
        assert!(generated_secret("9BAD", "$$cap_gen_random_hex(32)").is_err());
    }
}
