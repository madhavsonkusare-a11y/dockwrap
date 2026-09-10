//! Machine-readable adapter results; no installation or credential generation.
use local_store::importers::{caprover, runtipi};
use std::io::{self, Read};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin()
        .take(32 * 1024 * 1024 + 1)
        .read_to_string(&mut input)?;
    if input.len() > 32 * 1024 * 1024 {
        return Err("input too large".into());
    }
    let records: Vec<serde_json::Value> = serde_json::from_str(&input)?;
    let mut output = Vec::new();
    for record in records {
        let id = record["id"].as_str().ok_or("missing id")?;
        let source = record["source"].as_str().ok_or("missing source")?;
        let definition = record["definition"].to_string();
        let config = record.get("config").map(serde_json::Value::to_string);
        let result = match source {
            "caprover" => caprover::import(id, &definition),
            "runtipi" => runtipi::import(id, &definition, config.as_deref()),
            _ => return Err("unknown adapter".into()),
        };
        let mut row = serde_json::json!({"id":id,"source":source});
        match result {
            Err(_) => {
                row["importable"] = false.into();
                row["blockers"] = serde_json::json!([{"category":"parse","feature":"definition"}]);
            }
            Ok(outcome) => {
                row["importable"] = outcome.is_importable().into();
                row["blockers"] = outcome
                    .limitations
                    .iter()
                    // The detail is what makes a blocker actionable. Without
                    // it "plan policy: 25 apps" is a number; with it the list
                    // separates a rule worth relaxing from one worth keeping.
                    .map(|l| {
                        serde_json::json!({
                            "category": l.category(),
                            "feature": l.feature(),
                            "detail": l.detail(),
                        })
                    })
                    .collect();
                if let Some(template) = outcome.template {
                    row["service_count"] = template.plan.services.len().into();
                    row["images"] = template
                        .plan
                        .services
                        .iter()
                        .map(|s| serde_json::Value::String(s.image.clone()))
                        .collect();
                    row["primary_image"] = template
                        .plan
                        .published()
                        .map(|(s, _)| s.image.clone())
                        .into();
                    row["fields"] = template.fields.iter().map(|f| serde_json::json!({"key":f.key,"required":f.required,"has_default":f.default.is_some(),"sensitive":f.sensitive})).collect();
                    row["required_inputs"] = template
                        .fields
                        .iter()
                        .filter(|f| f.required && f.default.is_none())
                        .count()
                        .into();
                    row["generated_credentials"] = template.secrets.len().into();
                }
            }
        }
        output.push(row);
    }
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
