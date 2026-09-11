//! Typed setup fields and generated secrets.
//!
//! Most upstream definitions cannot be installed as they stand because their
//! environment carries `${PLACEHOLDER}` values: some are answers a person must
//! give, others are credentials that should be generated. The Runtipi import
//! report counts 181 of 250 apps blocked on exactly this.
//!
//! A [`PlanTemplate`] pairs a plan holding those placeholders with the fields
//! and secrets that fill them. Resolution is all-or-nothing: a placeholder with
//! no declared source is an error, never a literal `${...}` shipped into a
//! container.
use crate::plan::DeploymentPlan;
use std::collections::BTreeMap;
mod review;
pub use review::{SetupFieldReview, SetupReview};

/// Characters used for generated secrets.
///
/// Letters and digits only: the value travels through YAML, a Compose
/// environment and often a connection string, and punctuation invites quoting
/// bugs in all three. Length carries the strength instead.
const SECRET_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
/// Largest multiple of the alphabet length that fits in a byte, used to reject
/// samples that would otherwise make early characters more likely.
const UNBIASED_LIMIT: u8 = 248; // 62 * 4

/// Cap the source, compiled program, DFA cache and nesting; no backtracking
/// engine. This is not a JavaScript evaluator and does not support lookaround.
pub fn compile_setup_pattern(pattern: &str) -> Result<regex::bytes::Regex, String> {
    if pattern.len() > 2048 {
        return Err("Setup validation pattern is too long".into());
    }
    regex::bytes::RegexBuilder::new(pattern)
        .unicode(false)
        .size_limit(256 * 1024)
        .dfa_size_limit(256 * 1024)
        .nest_limit(32)
        .build()
        .map_err(|_| "Setup validation pattern uses unsupported syntax or exceeds limits".into())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldKind {
    Text {
        min_len: usize,
        max_len: usize,
    },
    /// ASCII-only pattern subset for upstream JavaScript validation rules.
    /// Importers must refuse unsupported syntax, never silently omit a rule.
    Pattern {
        pattern: String,
        min_len: usize,
        max_len: usize,
    },
    Number {
        min: i64,
        max: i64,
    },
    Boolean,
    Choice {
        options: Vec<String>,
    },
    /// A folder on this computer the person chooses to share with the app.
    ///
    /// The only answer that hands a container something outside the storage
    /// this product manages. `crate::folders::share_folder` decides whether a
    /// given folder is allowed; `read_only` travels with the field so the
    /// review can say plainly whether the app may change what is in there.
    Folder {
        read_only: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupField {
    /// Placeholder name, which is also the environment key it fills.
    pub key: String,
    /// What to call it in the interface.
    pub label: String,
    pub kind: FieldKind,
    pub required: bool,
    pub default: Option<String>,
    /// A value the user supplies that is still a credential — an API key, say.
    /// Interfaces must mask it and diagnostics must not echo it. Distinct from
    /// a [`SecretSpec`], which is generated rather than asked for.
    pub sensitive: bool,
}

/// A credential generated at install time rather than asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretSpec {
    pub key: String,
    /// Output characters, not random-byte count.
    pub length: usize,
    pub format: SecretFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretFormat {
    Alphanumeric,
    Hex,
    /// `length` random *bytes*, base64-encoded with padding — Runtipi's
    /// `encoding: base64`. An AES-256 key is 32 bytes, which is 44 characters
    /// here; 32 alphanumeric characters decode to 24 bytes and the app refuses
    /// the key.
    Base64,
}

impl SecretSpec {
    pub fn generate(&self) -> Result<String, String> {
        generate_secret_with_format(self.length, self.format)
    }
    /// How many characters a generated value has.
    pub fn output_len(&self) -> usize {
        match self.format {
            SecretFormat::Base64 => 4 * self.length.div_ceil(3),
            _ => self.length,
        }
    }
    fn accepts(&self, value: &str) -> bool {
        value.len() == self.output_len()
            && value.bytes().all(|byte| match self.format {
                SecretFormat::Alphanumeric => byte.is_ascii_alphanumeric(),
                SecretFormat::Hex => byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte),
                SecretFormat::Base64 => {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'=')
                }
            })
    }
}

/// Standard base64 with padding, for the few bytes a secret needs.
fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(4 * bytes.len().div_ceil(3));
    for chunk in bytes.chunks(3) {
        let n = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        for (index, shift) in [18, 12, 6, 0].into_iter().enumerate() {
            if index <= chunk.len() {
                out.push(TABLE[((n >> shift) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// Placeholders the installer fills, rather than the person installing.
///
/// Many upstream definitions tell an app its own address, because the platform
/// they were written for hands every app a public hostname. This project hands
/// it a loopback address instead — but not until the host port is settled,
/// which happens during installation and can differ from what the plan asked
/// for. Baking an address in at import time was how an app came to be told it
/// lives somewhere it does not.
pub const PLATFORM_URL: &str = "LOCAL_STORE_URL";
pub const PLATFORM_HOST: &str = "LOCAL_STORE_HOST";
pub const PLATFORM_PORT: &str = "LOCAL_STORE_PORT";
const PLATFORM_KEYS: &[&str] = &[PLATFORM_URL, PLATFORM_HOST, PLATFORM_PORT];

/// Whether this key is supplied by the installer rather than declared.
pub fn is_platform_key(key: &str) -> bool {
    PLATFORM_KEYS.contains(&key)
}

/// A service's second address, as its URL and as its bare port. The suffix
/// is the service name in capitals: `realtime` becomes
/// `LOCAL_STORE_URL_REALTIME`.
const COMPANION_URL: &str = "LOCAL_STORE_URL_";
const COMPANION_PORT: &str = "LOCAL_STORE_PORT_";

/// The service suffix a second-address placeholder names, if it is one.
pub fn companion_service(key: &str) -> Option<&str> {
    key.strip_prefix(COMPANION_URL)
        .or_else(|| key.strip_prefix(COMPANION_PORT))
        .filter(|suffix| !suffix.is_empty())
}

/// How a service name appears in its second-address placeholders.
pub fn companion_suffix(service: &str) -> String {
    service.to_ascii_uppercase().replace('-', "_")
}

/// Whether a declared field or secret would collide with what the installer
/// supplies.
fn is_reserved_key(key: &str) -> bool {
    PLATFORM_KEYS.contains(&key) || companion_service(key).is_some()
}

/// Replace the platform placeholders now that the address is known.
///
/// Called once, during installation, after the host port has been chosen and
/// before any answer is resolved, so nothing downstream ever sees one of these.
///
/// A second address is filled from the plan itself, so its port has to be
/// settled on the plan before this runs, the same as the main one.
pub fn fill_platform_values(plan: &mut crate::plan::DeploymentPlan, host_port: u16) {
    let authority = format!("localhost:{host_port}");
    let mut values = vec![
        (PLATFORM_URL.to_owned(), format!("http://{authority}")),
        (PLATFORM_HOST.to_owned(), authority),
        (PLATFORM_PORT.to_owned(), host_port.to_string()),
    ];
    for (service, port) in plan.companions() {
        let suffix = companion_suffix(&service.name);
        values.push((
            format!("{COMPANION_URL}{suffix}"),
            format!("http://localhost:{}", port.host),
        ));
        values.push((format!("{COMPANION_PORT}{suffix}"), port.host.to_string()));
    }
    for service in &mut plan.services {
        for (_, value) in &mut service.environment {
            for (key, filled) in &values {
                let placeholder = format!("${{{key}}}");
                if value.contains(&placeholder) {
                    *value = value.replace(&placeholder, filled);
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanTemplate {
    /// A plan whose environment values may contain `${KEY}` placeholders.
    pub plan: DeploymentPlan,
    pub fields: Vec<SetupField>,
    pub secrets: Vec<SecretSpec>,
    /// Files an app expects to find in its data folder the first time it
    /// starts — an nginx config, a starter settings file, SQL to initialise a
    /// database. Runtipi ships these beside a definition and copies them in
    /// on install; without them a mount of `data/proxy/nginx.conf` becomes an
    /// empty directory and the app fails to start.
    pub seeds: Vec<SeedFile>,
    /// How long the first start may take, when a review found an app needs
    /// longer than the default. Khoj downloads its embedding models before
    /// it answers — 130 seconds when measured, and more on a slower line.
    pub first_start: Option<std::time::Duration>,
}

/// One file written into an app's data folder before it first starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedFile {
    /// Relative to the app's folder, and always inside `data/`.
    pub path: String,
    pub content: String,
}

/// How many seed files, and how large, a template may carry. A seed is a
/// starting configuration, not a way to ship an application.
pub const MAX_SEEDS: usize = 64;
pub const MAX_SEED_BYTES: usize = 256 * 1024;

/// Whether a seed path stays inside the app's data folder.
pub fn is_confined_seed_path(path: &str) -> bool {
    path.starts_with("data/")
        && !path.ends_with('/')
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
        && path
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'))
}

/// What is wrong with one answer, in terms that can be shown against a field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldError {
    pub key: String,
    pub message: String,
}

impl SetupField {
    /// Check one answer and return the value to use.
    ///
    /// A missing optional field falls back to its default; a missing required
    /// field is an error rather than an empty string, because an empty value
    /// usually reaches the container as a real setting.
    pub fn accept(&self, answer: Option<&str>) -> Result<String, FieldError> {
        let fail = |message: &str| FieldError {
            key: self.key.clone(),
            message: message.to_owned(),
        };
        let given = answer.map(str::trim).filter(|value| !value.is_empty());
        let value = match (given, self.default.as_deref()) {
            (Some(value), _) => value.to_owned(),
            (None, Some(default)) => default.to_owned(),
            (None, None) if self.required => return Err(fail("This is required.")),
            (None, None) => return Ok(String::new()),
        };
        if value.chars().any(char::is_control) {
            return Err(fail("Remove line breaks and control characters."));
        }
        match &self.kind {
            FieldKind::Pattern {
                pattern,
                min_len,
                max_len,
            } => {
                if !value.is_ascii() || value.len() < *min_len || value.len() > *max_len {
                    return Err(fail(&format!(
                        "Use between {min_len} and {max_len} ASCII characters."
                    )));
                }
                let regex = compile_setup_pattern(pattern)
                    .map_err(|_| fail("The app's validation rule is unsupported."))?;
                if !regex.is_match(value.as_bytes()) {
                    return Err(fail("This value does not match the app's required format."));
                }
            }
            FieldKind::Text { min_len, max_len } => {
                let length = value.chars().count();
                if length < *min_len || length > *max_len {
                    return Err(fail(&format!(
                        "Use between {min_len} and {max_len} characters."
                    )));
                }
            }
            FieldKind::Number { min, max } => {
                let number: i64 = value.parse().map_err(|_| fail("Enter a whole number."))?;
                if number < *min || number > *max {
                    return Err(fail(&format!("Enter a number between {min} and {max}.")));
                }
            }
            FieldKind::Boolean => {
                if !matches!(value.as_str(), "true" | "false") {
                    return Err(fail("Choose either true or false."));
                }
            }
            FieldKind::Choice { options } => {
                if !options.iter().any(|option| option == &value) {
                    return Err(fail(&format!("Choose one of: {}.", options.join(", "))));
                }
            }
            FieldKind::Folder { read_only } => {
                // Checked here rather than at install time, so a folder that
                // cannot be shared is refused while the person is still
                // looking at the form.
                let shared = crate::folders::share_folder(
                    &value,
                    *read_only,
                    // The managed apps root's parent is Local Store's own
                    // configuration directory, which must never be shared.
                    crate::storage::managed_apps_root()
                        .parent()
                        .unwrap_or(std::path::Path::new("")),
                )
                .map_err(|refusal| fail(&refusal.0))?;
                // The answer becomes the resolved path, so what is mounted is
                // what was judged rather than what was typed.
                return Ok(shared.path.to_string_lossy().into_owned());
            }
        }
        Ok(value)
    }
}

/// Generate one secret from the operating system's random source.
///
/// Rejection sampling keeps every character equally likely; taking `byte % 62`
/// directly would quietly favour the first few letters of the alphabet.
pub fn generate_secret(length: usize) -> Result<String, String> {
    generate_secret_with_format(length, SecretFormat::Alphanumeric)
}

/// The characters [`generate`](SecretSpec::generate) draws from. A
/// compatibility proof has to reason about this set, so it is named once here
/// rather than restated wherever it is needed.
fn alphabet(format: SecretFormat) -> &'static [u8] {
    match format {
        SecretFormat::Alphanumeric => SECRET_ALPHABET,
        SecretFormat::Hex => b"0123456789abcdef",
        SecretFormat::Base64 => {
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/="
        }
    }
}

/// Does every character this format generates satisfy the class `atom`?
///
/// `None` means the shape was not recognized, which callers must treat as
/// unproven rather than as a negative answer.
fn class_accepts_all(atom: &str, alphabet: &[u8]) -> Option<bool> {
    if atom == "." {
        // `.` excludes only newline, which no alphabet here contains.
        return Some(true);
    }
    if let Some(inner) = atom.strip_prefix('(').and_then(|a| a.strip_suffix(')')) {
        // A union accepts everything if any one branch does. Branches that
        // only cover the alphabet between them are left unproven on purpose.
        let mut recognized = false;
        for branch in inner.split('|') {
            match class_accepts_all(branch, alphabet) {
                Some(true) => return Some(true),
                Some(false) => recognized = true,
                None => {}
            }
        }
        return recognized.then_some(false);
    }
    if let Some(body) = atom.strip_prefix('[').and_then(|a| a.strip_suffix(']')) {
        let (negated, body) = match body.strip_prefix('^') {
            Some(rest) => (true, rest),
            None => (false, body),
        };
        for byte in alphabet {
            let present = set_contains(body, *byte)?;
            if present == negated {
                return Some(false);
            }
        }
        return Some(true);
    }
    if atom.len() == 2 && atom.starts_with('\\') {
        let class = atom.as_bytes()[1];
        return alphabet.iter().try_fold(true, |all, byte| {
            Some(all && escape_contains(class, *byte)?)
        });
    }
    None
}

/// Whether the shorthand class `\d`, `\w` or `\s` contains `byte`.
fn escape_contains(class: u8, byte: u8) -> Option<bool> {
    match class {
        b'd' => Some(byte.is_ascii_digit()),
        b'w' => Some(byte.is_ascii_alphanumeric() || byte == b'_'),
        b's' => Some(byte.is_ascii_whitespace()),
        _ => None,
    }
}

/// Whether a bracketed set body contains `byte`. `None` for any syntax this
/// deliberately narrow reader does not recognize.
fn set_contains(body: &str, byte: u8) -> Option<bool> {
    let bytes = body.as_bytes();
    let mut index = 0;
    let mut found = false;
    while index < bytes.len() {
        let current = bytes[index];
        if current == b'\\' {
            let escaped = *bytes.get(index + 1)?;
            index += 2;
            if let Some(inside) = escape_contains(escaped, byte) {
                found |= inside;
                continue;
            }
            // A punctuation escape stands for that literal character.
            if !escaped.is_ascii_alphanumeric() {
                found |= escaped == byte;
                continue;
            }
            // `\D`, `\W`, `\S` and anything else: not recognized.
            return None;
        }
        // A range, but only when the dash sits between two plain characters.
        if index + 2 < bytes.len() && bytes[index + 1] == b'-' && bytes[index + 2] != b'\\' {
            let (low, high) = (current, bytes[index + 2]);
            if low > high {
                return None;
            }
            found |= (low..=high).contains(&byte);
            index += 3;
            continue;
        }
        found |= current == byte;
        index += 1;
    }
    Some(found)
}

/// Split a trailing quantifier off `body`, returning the atom and the number
/// of characters it may consume.
fn split_quantifier(body: &str) -> Option<(&str, usize, Option<usize>)> {
    if let Some(atom) = body.strip_suffix('+') {
        return Some((atom, 1, None));
    }
    if let Some(atom) = body.strip_suffix('*') {
        return Some((atom, 0, None));
    }
    if let Some(atom) = body.strip_suffix('?') {
        return Some((atom, 0, Some(1)));
    }
    if let Some(rest) = body.strip_suffix('}') {
        let open = rest.rfind('{')?;
        let (atom, counts) = rest.split_at(open);
        let counts = &counts[1..];
        let (low, high) = match counts.split_once(',') {
            None => {
                let exact: usize = counts.parse().ok()?;
                (exact, Some(exact))
            }
            Some((low, "")) => (low.parse().ok()?, None),
            Some((low, high)) => (low.parse().ok()?, Some(high.parse().ok()?)),
        };
        if high.is_some_and(|high| high < low) {
            return None;
        }
        return Some((atom, low, high));
    }
    Some((body, 1, Some(1)))
}

impl SecretSpec {
    /// Whether every value this spec can generate satisfies `pattern`.
    ///
    /// Upstream catalogues attach a validation rule to a generated credential,
    /// and the rule was written for a value a person types. Accepting the pair
    /// without checking risks handing someone a credential their own app then
    /// refuses, which surfaces as an app that cannot open its own data.
    ///
    /// This is sound by construction and deliberately incomplete. It decides
    /// one narrow shape — a single character class carrying a single
    /// quantifier, optionally anchored — and answers `false` for everything
    /// else. A wrong `true` here would be a credential failure in someone's
    /// hands, so anything not proven is reported as unproven.
    pub fn every_value_matches(&self, pattern: &str) -> bool {
        let anchored_start = pattern.starts_with('^');
        let body = pattern.strip_prefix('^').unwrap_or(pattern);
        // A `$` that is itself escaped is a literal, not an anchor.
        let anchored_end = body.ends_with('$') && !body.ends_with("\\$");
        let body = if anchored_end {
            &body[..body.len() - 1]
        } else {
            body
        };

        let Some((atom, low, high)) = split_quantifier(body) else {
            return false;
        };
        if class_accepts_all(atom, alphabet(self.format)) != Some(true) {
            return false;
        }
        if self.output_len() < low {
            return false;
        }
        // Only a rule anchored at both ends has to consume the whole value;
        // otherwise matching some run of characters inside it is enough, and
        // every character is already known to satisfy the class.
        if anchored_start && anchored_end {
            if let Some(high) = high {
                if self.output_len() > high {
                    return false;
                }
            }
        }
        true
    }
}

pub fn generate_secret_with_format(length: usize, format: SecretFormat) -> Result<String, String> {
    if !(16..=256).contains(&length) {
        return Err("secret length must be between 16 and 256 characters".into());
    }
    if format == SecretFormat::Base64 {
        let mut bytes = vec![0_u8; length];
        getrandom::fill(&mut bytes)
            .map_err(|error| format!("could not read the system random source: {error}"))?;
        return Ok(base64_encode(&bytes));
    }
    let mut secret = String::with_capacity(length);
    let alphabet = alphabet(format);
    let limit = match format {
        SecretFormat::Alphanumeric => UNBIASED_LIMIT as u16,
        SecretFormat::Hex => 256,
        SecretFormat::Base64 => unreachable!("base64 secrets return above"),
    };
    let mut buffer = [0_u8; 64];
    while secret.len() < length {
        getrandom::fill(&mut buffer)
            .map_err(|error| format!("could not read the system random source: {error}"))?;
        for byte in buffer {
            if secret.len() == length {
                break;
            }
            if u16::from(byte) < limit {
                let index = usize::from(byte) % alphabet.len();
                secret.push(char::from(alphabet[index]));
            }
        }
    }
    Ok(secret)
}

impl PlanTemplate {
    pub fn validate(&self) -> Result<(), String> {
        if self.seeds.len() > MAX_SEEDS {
            return Err(format!(
                "{} seed files is more than the {MAX_SEEDS} allowed",
                self.seeds.len()
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        for seed in &self.seeds {
            if !is_confined_seed_path(&seed.path) {
                return Err(format!(
                    "seed file {:?} is not a plain path inside data/",
                    seed.path
                ));
            }
            if seed.content.len() > MAX_SEED_BYTES {
                return Err(format!(
                    "seed file {:?} is larger than {MAX_SEED_BYTES} bytes",
                    seed.path
                ));
            }
            if !seen.insert(seed.path.as_str()) {
                return Err(format!("seed file {:?} is declared twice", seed.path));
            }
        }
        for field in &self.fields {
            if let FieldKind::Pattern {
                pattern,
                min_len,
                max_len,
            } = &field.kind
            {
                if min_len > max_len || *max_len > 4096 {
                    return Err("Invalid pattern field length bounds".into());
                }
                compile_setup_pattern(pattern)?;
            }
            if !is_key(&field.key) {
                return Err(format!(
                    "setup field key {:?} is not NAME_LIKE_THIS",
                    field.key
                ));
            }
            if is_reserved_key(&field.key) {
                return Err(format!(
                    "setup field key {:?} is reserved for the installer",
                    field.key
                ));
            }
            if let FieldKind::Choice { options } = &field.kind {
                if options.is_empty() {
                    return Err(format!("field {:?} offers no choices", field.key));
                }
            }
            // A default that its own field would reject is a trap: the install
            // fails only for the user who leaves the field alone.
            if field.default.is_some() {
                field.accept(None).map_err(|error| {
                    format!("default for {:?} is invalid: {}", field.key, error.message)
                })?;
            }
        }
        for secret in &self.secrets {
            if !(16..=256).contains(&secret.length) {
                return Err(format!(
                    "secret {:?} length must be between 16 and 256 characters",
                    secret.key
                ));
            }
            if !is_key(&secret.key) {
                return Err(format!("secret key {:?} is not NAME_LIKE_THIS", secret.key));
            }
            if is_reserved_key(&secret.key) {
                return Err(format!(
                    "secret key {:?} is reserved for the installer",
                    secret.key
                ));
            }
            if self.fields.iter().any(|field| field.key == secret.key) {
                return Err(format!(
                    "{:?} is declared as both a setup field and a secret",
                    secret.key
                ));
            }
        }
        // Every placeholder must have a source, and every source must be used.
        let declared: Vec<&str> = self
            .fields
            .iter()
            .map(|field| field.key.as_str())
            .chain(self.secrets.iter().map(|secret| secret.key.as_str()))
            .collect();
        let mut used = vec![false; declared.len()];
        // A folder answer is referenced by a mount rather than by an
        // environment value. Without this it reads as declared-but-unused and
        // the whole template is refused — the same way a time zone field
        // nothing referenced used to be dropped as inert.
        for service in &self.plan.services {
            for mount in &service.mounts {
                let crate::plan::PlanMount::Host { source, .. } = mount else {
                    continue;
                };
                for placeholder in placeholders(source) {
                    match declared.iter().position(|key| *key == placeholder.key) {
                        Some(index) => used[index] = true,
                        None => {
                            return Err(format!(
                            "a shared folder mount refers to {:?}, which no setup field declares",
                            placeholder.key
                        ))
                        }
                    }
                }
            }
        }
        let companions: Vec<String> = self
            .plan
            .companions()
            .iter()
            .map(|(service, _)| companion_suffix(&service.name))
            .collect();
        for (_, value) in self.environment() {
            for placeholder in placeholders(value) {
                if PLATFORM_KEYS.contains(&placeholder.key) {
                    // Supplied by the installer, so it needs no declaration
                    // here and nobody is ever asked for it.
                    continue;
                }
                if let Some(wanted) = companion_service(placeholder.key) {
                    // Supplied too, but only for a service that has one: a
                    // placeholder naming any other would never be filled.
                    if companions.iter().any(|suffix| suffix == wanted) {
                        continue;
                    }
                    return Err(format!(
                        "placeholder {:?} names a second address no service publishes",
                        placeholder.key
                    ));
                }
                match declared.iter().position(|key| *key == placeholder.key) {
                    Some(index) => used[index] = true,
                    None => {
                        return Err(format!(
                            "placeholder {:?} has no setup field or secret",
                            placeholder.key
                        ))
                    }
                }
            }
        }
        if let Some(index) = used.iter().position(|used| !used) {
            return Err(format!(
                "{:?} is declared but no service uses it",
                declared[index]
            ));
        }
        // Importability includes topology and deployment policy, not only
        // whether every setup placeholder has a source.
        self.plan.validate()
    }

    fn environment(&self) -> impl Iterator<Item = (&str, &str)> {
        self.plan
            .services
            .iter()
            .flat_map(|service| &service.environment)
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }

    /// Check every answer, collecting all problems rather than only the first.
    pub fn accept_answers(
        &self,
        answers: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, Vec<FieldError>> {
        let mut accepted = BTreeMap::new();
        let mut errors = Vec::new();
        for field in &self.fields {
            match field.accept(answers.get(&field.key).map(String::as_str)) {
                Ok(value) => {
                    accepted.insert(field.key.clone(), value);
                }
                Err(error) => errors.push(error),
            }
        }
        // An answer nobody asked for usually means a renamed field, which would
        // otherwise be silently dropped.
        for key in answers.keys() {
            if !self.fields.iter().any(|field| &field.key == key) {
                errors.push(FieldError {
                    key: key.clone(),
                    message: "This app has no such setup field.".into(),
                });
            }
        }
        if errors.is_empty() {
            Ok(accepted)
        } else {
            Err(errors)
        }
    }

    pub fn generate_secrets(&self) -> Result<BTreeMap<String, String>, String> {
        self.secrets
            .iter()
            .map(|secret| Ok((secret.key.clone(), secret.generate()?)))
            .collect()
    }

    /// Fill every placeholder and produce an installable plan.
    ///
    /// Existing secrets are reused when supplied, which is what lets a
    /// reinstall over preserved data keep working: regenerating a database
    /// password would leave the app unable to open its own data.
    pub fn resolve(
        &self,
        answers: &BTreeMap<String, String>,
        secrets: &BTreeMap<String, String>,
    ) -> Result<DeploymentPlan, String> {
        self.validate()?;
        let accepted = self.accept_answers(answers).map_err(|errors| {
            let mut listed: Vec<String> = errors
                .iter()
                .map(|error| format!("{}: {}", error.key, error.message))
                .collect();
            listed.sort();
            listed.join("; ")
        })?;
        for secret in &self.secrets {
            if !secrets.contains_key(&secret.key) {
                return Err(format!("no value supplied for secret {:?}", secret.key));
            }
            if !secret.accepts(&secrets[&secret.key]) {
                return Err(format!("stored secret {:?} does not match its required length or format; restore compatible credentials before reinstalling", secret.key));
            }
        }

        let mut resolved = self.plan.clone();
        for service in &mut resolved.services {
            for (key, value) in &mut service.environment {
                let filled = substitute(value, &accepted, secrets).map_err(|missing| {
                    // Never echo the value: it may be half-substituted and
                    // carrying a secret.
                    format!("{key} needs a value for {missing:?}")
                })?;
                *value = filled;
            }
        }
        // A shared folder is an answer like any other, but it lands in a mount
        // rather than in the environment. Doing it here means the plan that
        // reaches Docker carries the path `share_folder` approved, and the
        // validation below sees a resolved path rather than a placeholder.
        for service in &mut resolved.services {
            for mount in &mut service.mounts {
                if let crate::plan::PlanMount::Host { source, .. } = mount {
                    *source = substitute(source, &accepted, secrets)
                        .map_err(|missing| format!("no folder was chosen for {missing:?}"))?;
                }
            }
        }
        resolved.validate()?;
        Ok(resolved)
    }
}

struct Placeholder<'a> {
    key: &'a str,
    default: Option<&'a str>,
    span: (usize, usize),
}

/// Find `${KEY}` and `${KEY:-default}` occurrences.
fn placeholders(value: &str) -> Vec<Placeholder<'_>> {
    let mut found = Vec::new();
    let bytes = value.as_bytes();
    let mut index = 0;
    while let Some(start) = value[index..].find("${") {
        let open = index + start;
        let Some(close) = value[open..].find('}') else {
            break;
        };
        let end = open + close + 1;
        let inner = &value[open + 2..end - 1];
        let (key, default) = match inner.split_once(":-") {
            Some((key, default)) => (key, Some(default)),
            None => (inner, None),
        };
        if is_key(key) {
            found.push(Placeholder {
                key,
                default,
                span: (open, end),
            });
        }
        index = end;
        if index >= bytes.len() {
            break;
        }
    }
    found
}

fn substitute(
    value: &str,
    answers: &BTreeMap<String, String>,
    secrets: &BTreeMap<String, String>,
) -> Result<String, String> {
    let found = placeholders(value);
    if found.is_empty() {
        return Ok(value.to_owned());
    }
    let mut out = String::with_capacity(value.len());
    let mut cursor = 0;
    for placeholder in found {
        out.push_str(&value[cursor..placeholder.span.0]);
        let replacement = answers
            .get(placeholder.key)
            .or_else(|| secrets.get(placeholder.key))
            .map(String::as_str)
            .or(placeholder.default)
            .ok_or_else(|| placeholder.key.to_owned())?;
        out.push_str(replacement);
        cursor = placeholder.span.1;
    }
    out.push_str(&value[cursor..]);
    Ok(out)
}

fn is_key(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

#[cfg(test)]
mod tests {
    /// RFC 4648's own examples, and a key of the size Plausible and BookStack
    /// ask for: 32 random bytes, which is 44 characters with padding.
    #[test]
    fn second_address_placeholders_are_the_installers_to_fill() {
        assert_eq!(
            companion_service("LOCAL_STORE_URL_REALTIME"),
            Some("REALTIME")
        );
        assert_eq!(
            companion_service("LOCAL_STORE_PORT_OBJECT_STORE"),
            Some("OBJECT_STORE")
        );
        assert_eq!(companion_service("LOCAL_STORE_URL"), None);
        assert_eq!(companion_service("LOCAL_STORE_URL_"), None);
        assert_eq!(companion_suffix("object-store"), "OBJECT_STORE");
        assert!(is_reserved_key("LOCAL_STORE_URL_REALTIME"));
        assert!(!is_reserved_key("REALTIME_URL"));
    }

    #[test]
    fn base64_secrets_are_random_bytes_encoded_as_runtipi_does() {
        for (bytes, encoded) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(super::base64_encode(bytes.as_bytes()), encoded);
        }
        let spec = SecretSpec {
            key: "KEY".into(),
            length: 32,
            format: SecretFormat::Base64,
        };
        let value = spec.generate().unwrap();
        assert_eq!(value.len(), 44, "{value}");
        assert!(
            value.ends_with('='),
            "32 bytes need one padding character: {value}"
        );
        assert!(spec.accepts(&value));
        assert!(
            !spec.accepts(&"a".repeat(32)),
            "an alphanumeric key of the old shape was accepted"
        );
    }

    use super::*;

    fn hex(length: usize) -> SecretSpec {
        SecretSpec {
            key: "CAP_SECRET".into(),
            length,
            format: SecretFormat::Hex,
        }
    }

    /// Every rule that actually appears beside a generated credential in the
    /// pinned CapRover catalogue, translated the way the importer translates
    /// it. The proof claims all of these accept any value we generate, so the
    /// claim is checked against a great many values we actually generate.
    #[test]
    fn a_proven_rule_accepts_every_value_the_generator_can_produce() {
        let proven: [(&str, usize); 8] = [
            (r".{1,}", 16),
            (r"^([^\s^/])+$", 16),
            (r"^[^@]{12,}$", 16),
            (r".{8,}", 16),
            (r#"^(\w|[^\s"'\\])+$"#, 32),
            (r"^[a-fA-F0-9]+$", 32),
            (r".{12,20}", 32),
            (r"^[^@]{26,}$", 32),
        ];
        for (pattern, length) in proven {
            let spec = hex(length);
            assert!(
                spec.every_value_matches(pattern),
                "{pattern} should be provable"
            );
            let regex = compile_setup_pattern(pattern).expect("pattern should compile");
            // A proof about every value is worth little if no real value is
            // ever put to it.
            for _ in 0..2000 {
                let value = spec.generate().unwrap();
                assert!(
                    regex.is_match(value.as_bytes()),
                    "{pattern} rejected a value the proof said it would accept"
                );
            }
        }
    }

    #[test]
    fn a_rule_that_cannot_be_proven_is_reported_as_unproven() {
        for (pattern, length) in [
            // A hex value can be all letters, so all-digits is not provable.
            (r"^\d+$", 16),
            // Only letters a-f: a hex value can contain digits.
            (r"^[a-f]+$", 16),
            // Anchored at both ends and shorter than the value.
            (r"^.{1,8}$", 16),
            (r"^[^@]{26,}$", 16),
            // Shapes this deliberately narrow reader does not decide.
            (r"^\w+-\w+$", 16),
            (r"^(abc)+$", 16),
            (r"^(?:x)+$", 16),
            (r"^\W+$", 16),
            (r"", 16),
        ] {
            assert!(
                !hex(length).every_value_matches(pattern),
                "{pattern} must not be claimed as proven"
            );
        }
    }

    /// The alphanumeric generator is a different alphabet, and a rule proven
    /// for one must not be assumed for the other.
    #[test]
    fn a_proof_is_specific_to_the_alphabet_it_was_made_for() {
        let alphanumeric = SecretSpec {
            key: "CAP_SECRET".into(),
            length: 32,
            format: SecretFormat::Alphanumeric,
        };
        // Hexadecimal is a subset of the hex class; alphanumeric is not.
        assert!(hex(32).every_value_matches(r"^[a-fA-F0-9]+$"));
        assert!(!alphanumeric.every_value_matches(r"^[a-fA-F0-9]+$"));
        // Both are word characters, so this holds for either.
        assert!(hex(32).every_value_matches(r"^\w+$"));
        assert!(alphanumeric.every_value_matches(r"^\w+$"));
    }
    use crate::plan::{PlanService, PublishedPort};
    use std::collections::BTreeSet;

    fn template() -> PlanTemplate {
        PlanTemplate {
            first_start: None,
            seeds: Vec::new(),
            plan: DeploymentPlan {
                id: "example".into(),
                services: vec![PlanService {
                    name: "example".into(),
                    image: "example/app:1.0.0".into(),
                    digest: None,
                    environment: vec![
                        ("ADMIN_EMAIL".into(), "${ADMIN_EMAIL}".into()),
                        ("DB_PASSWORD".into(), "${DB_PASSWORD}".into()),
                        ("SITE_URL".into(), "http://localhost:${PORT:-8080}".into()),
                    ],
                    companion: None,
                    published: Some(PublishedPort {
                        host: 8080,
                        container: 8080,
                    }),
                    mounts: Vec::new(),
                    depends_on: Vec::new(),
                    overrides: crate::plan::PlanOverrides::default(),
                }],
                named_volumes: Vec::new(),
            },
            fields: vec![
                SetupField {
                    key: "ADMIN_EMAIL".into(),
                    label: "Administrator email".into(),
                    kind: FieldKind::Text {
                        min_len: 3,
                        max_len: 120,
                    },
                    required: true,
                    default: None,
                    sensitive: false,
                },
                SetupField {
                    key: "PORT".into(),
                    label: "Port".into(),
                    kind: FieldKind::Number {
                        min: 1024,
                        max: 65535,
                    },
                    required: false,
                    default: Some("8080".into()),
                    sensitive: false,
                },
            ],
            secrets: vec![SecretSpec {
                key: "DB_PASSWORD".into(),
                length: 32,
                format: crate::setup::SecretFormat::Alphanumeric,
            }],
        }
    }

    fn answers(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect()
    }

    #[test]
    fn a_template_resolves_into_an_installable_plan() {
        let template = template();
        let secrets = template.generate_secrets().unwrap();
        let plan = template
            .resolve(&answers(&[("ADMIN_EMAIL", "me@example.com")]), &secrets)
            .expect("template should resolve");

        let environment = &plan.services[0].environment;
        assert_eq!(environment[0].1, "me@example.com");
        assert_eq!(environment[1].1, secrets["DB_PASSWORD"]);
        // The unanswered field fell back to its default, inside a larger string.
        assert_eq!(environment[2].1, "http://localhost:8080");
        // No placeholder survives into something that reaches a container.
        assert!(!plan.to_compose().unwrap().contains("${"));
    }

    #[test]
    fn a_reinstall_can_keep_the_secret_the_data_was_written_with() {
        let template = template();
        let original = template.generate_secrets().unwrap();
        let again = template
            .resolve(&answers(&[("ADMIN_EMAIL", "me@example.com")]), &original)
            .unwrap();
        // Regenerating here would leave the app unable to open its own data.
        assert_eq!(again.services[0].environment[1].1, original["DB_PASSWORD"]);
    }

    #[test]
    fn generated_secrets_are_unpredictable_and_shaped_for_yaml() {
        let first = generate_secret(32).unwrap();
        let second = generate_secret(32).unwrap();
        assert_eq!(first.chars().count(), 32);
        assert_ne!(first, second);
        assert!(first.chars().all(|c| c.is_ascii_alphanumeric()));

        // Every alphabet character should appear across enough draws; a biased
        // or truncated alphabet shows up here.
        let mut seen = BTreeSet::new();
        for _ in 0..40 {
            seen.extend(generate_secret(64).unwrap().chars());
        }
        assert_eq!(seen.len(), SECRET_ALPHABET.len(), "alphabet not fully used");

        assert!(generate_secret(8).is_err(), "too short to be a credential");
        assert!(generate_secret(1024).is_err());
    }

    #[test]
    fn hexadecimal_secrets_resolve_and_reuse_without_becoming_alphanumeric() {
        let mut template = template();
        template.secrets[0].format = SecretFormat::Hex;
        let secrets = template.generate_secrets().unwrap();
        let key = &secrets["DB_PASSWORD"];
        assert_eq!(key.len(), 32);
        assert!(key
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));
        let given = answers(&[("ADMIN_EMAIL", "me@example.com")]);
        let plan = template.resolve(&given, &secrets).unwrap();
        assert_eq!(&plan.services[0].environment[1].1, key);
        assert_eq!(template.resolve(&given, &secrets).unwrap(), plan);
        let mut bad = secrets.clone();
        let invalid = "z".repeat(32);
        bad.insert("DB_PASSWORD".into(), invalid.clone());
        let error = template.resolve(&given, &bad).unwrap_err();
        assert!(!error.contains(&invalid));
        assert!(error.contains("format"));
        template.secrets[0].length = 8;
        assert!(template.validate().unwrap_err().contains("length"));
    }

    #[test]
    fn pattern_fields_enforce_defaults_bounds_and_do_not_echo_answers() {
        let field = SetupField {
            key: "USER".into(),
            label: "User".into(),
            kind: FieldKind::Pattern {
                pattern: "^[a-z][a-z0-9_]+$".into(),
                min_len: 2,
                max_len: 32,
            },
            required: true,
            default: Some("local_user".into()),
            sensitive: true,
        };
        assert_eq!(field.accept(None).unwrap(), "local_user");
        assert_eq!(field.accept(Some("alice_1")).unwrap(), "alice_1");
        for input in ["1user", "Uppercase", "用户", "private-value!"] {
            let error = field.accept(Some(input)).unwrap_err();
            assert!(!error.message.contains(input));
        }
        assert!(compile_setup_pattern("(?=secret)").is_err());
        assert!(compile_setup_pattern(&"x".repeat(2049)).is_err());
        assert!(compile_setup_pattern("(abc)\\1").is_err());
        let mut template = template();
        template.fields[0].kind = FieldKind::Pattern {
            pattern: "[".into(),
            min_len: 1,
            max_len: 64,
        };
        assert!(template.validate().is_err());
    }

    #[test]
    fn typed_answers_are_checked_against_their_kind() {
        let template = template();
        let long = "x".repeat(200);
        for (given, expected) in [
            (vec![("ADMIN_EMAIL", "")], "required"),
            (vec![("ADMIN_EMAIL", "ab")], "between 3 and 120"),
            (vec![("ADMIN_EMAIL", long.as_str())], "between 3 and 120"),
            (
                vec![("ADMIN_EMAIL", "me@example.com"), ("PORT", "80")],
                "between 1024 and 65535",
            ),
            (
                vec![("ADMIN_EMAIL", "me@example.com"), ("PORT", "eighty")],
                "whole number",
            ),
            (
                vec![("ADMIN_EMAIL", "me@example.com"), ("NOPE", "x")],
                "no such setup field",
            ),
        ] {
            let error = template
                .resolve(&answers(&given), &template.generate_secrets().unwrap())
                .unwrap_err();
            assert!(error.contains(expected), "{given:?} gave {error:?}");
        }
    }

    #[test]
    fn every_problem_is_reported_at_once_rather_than_one_at_a_time() {
        let template = template();
        let errors = template
            .accept_answers(&answers(&[("PORT", "80"), ("NOPE", "x")]))
            .unwrap_err();
        let keys: BTreeSet<&str> = errors.iter().map(|error| error.key.as_str()).collect();
        assert!(keys.contains("ADMIN_EMAIL"), "{keys:?}");
        assert!(keys.contains("PORT"), "{keys:?}");
        assert!(keys.contains("NOPE"), "{keys:?}");
    }

    #[test]
    fn a_placeholder_with_no_source_is_refused_rather_than_shipped() {
        let mut template = template();
        template.plan.services[0]
            .environment
            .push(("STRAY".into(), "${MYSTERY}".into()));
        let error = template.validate().unwrap_err();
        assert!(error.contains("MYSTERY"), "{error}");

        // And the reverse: something declared that nothing consumes.
        let mut unused = super::tests::template();
        unused.fields.push(SetupField {
            key: "UNUSED".into(),
            label: "Unused".into(),
            kind: FieldKind::Boolean,
            required: false,
            default: Some("false".into()),
            sensitive: false,
        });
        assert!(unused.validate().unwrap_err().contains("UNUSED"));
    }

    #[test]
    fn a_secret_value_never_appears_in_an_error() {
        let template = template();
        let mut secrets = template.generate_secrets().unwrap();
        let value = secrets["DB_PASSWORD"].clone();
        secrets.remove("DB_PASSWORD");
        let error = template
            .resolve(&answers(&[("ADMIN_EMAIL", "me@example.com")]), &secrets)
            .unwrap_err();
        assert!(error.contains("DB_PASSWORD"));
        assert!(!error.contains(&value), "the secret leaked into an error");
    }

    #[test]
    fn a_default_its_own_field_would_reject_is_caught_before_install() {
        let mut template = template();
        template.fields[1].default = Some("80".into());
        let error = template.validate().unwrap_err();
        assert!(error.contains("PORT"), "{error}");
        assert!(error.contains("1024"), "{error}");
    }

    #[test]
    fn a_key_cannot_be_both_asked_for_and_generated() {
        let mut template = template();
        template.secrets.push(SecretSpec {
            key: "ADMIN_EMAIL".into(),
            length: 32,
            format: crate::setup::SecretFormat::Alphanumeric,
        });
        assert!(template.validate().unwrap_err().contains("both"));
    }
}
