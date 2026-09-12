//! Read normalized variables on stdin; never install or emit their values.
use local_store::importers::caprover::{generated_secret, setup_variable, validation_pattern};
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin()
        .take(8 * 1024 * 1024 + 1)
        .read_to_string(&mut input)?;
    if input.len() > 8 * 1024 * 1024 {
        return Err("input exceeds limit".into());
    }
    let apps: serde_json::Value = serde_json::from_str(&input)?;
    let mut patterns = [0usize; 2];
    let mut secrets = [0usize; 2];
    let mut affected = 0;
    let mut variables_mapped = [0usize; 2];
    let mut mapping_reasons = std::collections::BTreeMap::<String, usize>::new();
    let mut refused_apps = std::collections::BTreeMap::new();
    for (app, variables) in apps.as_object().ok_or("expected apps object")? {
        let mut refused = false;
        let mut app_refusals = [0usize; 2];
        for variable in variables.as_array().ok_or("expected variables array")? {
            match setup_variable(variable) {
                Ok(_) => variables_mapped[0] += 1,
                Err(reason) => {
                    variables_mapped[1] += 1;
                    *mapping_reasons.entry(reason).or_default() += 1;
                }
            }
            if let Some(pattern) = variable
                .get("validRegex")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                let rejected = validation_pattern(pattern).is_err();
                patterns[usize::from(rejected)] += 1;
                app_refusals[0] += usize::from(rejected);
                refused |= rejected;
            }
            if let Some(value) = variable
                .get("defaultValue")
                .and_then(|v| v.as_str())
                .filter(|s| s.contains("$$cap_gen_random_hex("))
            {
                let rejected = generated_secret("CAP_AUDIT_SECRET", value).is_err();
                secrets[usize::from(rejected)] += 1;
                app_refusals[1] += usize::from(rejected);
                refused |= rejected;
            }
        }
        affected += usize::from(refused);
        if refused {
            refused_apps.insert(app, app_refusals);
        }
    }
    println!(
        "{}",
        serde_json::json!({"patterns_supported": patterns[0], "patterns_refused": patterns[1], "secret_declarations_supported": secrets[0], "secret_declarations_refused": secrets[1], "apps_with_refused_primitives": affected, "refused_apps": refused_apps, "variables_mapped": variables_mapped[0], "variables_refused": variables_mapped[1], "mapping_reasons": mapping_reasons})
    );
    Ok(())
}
