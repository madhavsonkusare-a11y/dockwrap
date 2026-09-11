//! Everything about a candidate that only the importer knows.
//!
//! Generating a reviewed template is mostly mechanical, but two parts of it
//! cannot be read off a definition by eye: which setup fields the mapping
//! actually produces, and which images the resulting plan actually runs. Both
//! come from running the importer, so this prints them as JSON for the
//! generator to assemble around.
//!
//! It decides nothing. The review decisions — whether an answer is a
//! credential, what the risk notes say, whether the app is offered at all —
//! belong to a person.
use std::collections::BTreeMap;

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(source), Some(id)) = (args.next(), args.next()) else {
        eprintln!("usage: template_facts <runtipi|caprover> <app-id> <definition-path>");
        std::process::exit(2);
    };
    let Some(path) = args.next() else {
        eprintln!("a definition path is required");
        std::process::exit(2);
    };
    let definition = std::fs::read_to_string(&path).expect("the definition is readable");
    let config = std::fs::read_to_string(
        std::path::Path::new(&path)
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join("config.json"),
    )
    .ok();

    let outcome = match source.as_str() {
        "runtipi" => local_store::importers::runtipi::import(&id, &definition, config.as_deref()),
        "caprover" => local_store::importers::caprover::import(&id, &definition),
        other => {
            eprintln!("no importer named {other}");
            std::process::exit(2);
        }
    };
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(reason) => {
            eprintln!("{id}: {reason}");
            std::process::exit(1);
        }
    };
    let Some(template) = outcome.template else {
        eprintln!(
            "{id}: not importable ({})",
            outcome
                .limitations
                .iter()
                .map(|limit| limit.feature())
                .collect::<Vec<_>>()
                .join(", ")
        );
        std::process::exit(1);
    };

    // Field labels come from upstream and are what a review edits; the
    // sensitivity guess is the importer's, and saying which fields it guessed
    // about is the point of showing them.
    let fields: Vec<serde_json::Value> = template
        .fields
        .iter()
        .map(|field| {
            serde_json::json!({
                "key": field.key,
                "upstream_label": field.label,
                "importer_thinks_sensitive": field.sensitive,
                "required": field.required,
                "kind": format!("{:?}", field.kind).split(' ').next().unwrap_or("?"),
            })
        })
        .collect();

    let images: Vec<String> = template
        .plan
        .services
        .iter()
        .map(|service| service.image.clone())
        .collect();
    let published = template.plan.published();
    // What the installer will create and what deleting data will remove, which
    // is what `data_storage` has to describe truthfully.
    let managed: Vec<&str> = template.plan.data_directories();
    let shared: Vec<&str> = template
        .plan
        .services
        .iter()
        .flat_map(|service| &service.mounts)
        .filter_map(|mount| match mount {
            local_store::plan::PlanMount::Host { target, .. } => Some(target.as_str()),
            _ => None,
        })
        .collect();

    // Data can also live in a named volume, which is neither the managed
    // folder nor something a person chose, and has to be described as such.
    let volumes: Vec<&str> = template
        .plan
        .services
        .iter()
        .flat_map(|service| &service.mounts)
        .filter_map(|mount| match mount {
            local_store::plan::PlanMount::Volume { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();

    let (seeds, binary_seeds) = if source == "runtipi" {
        local_store::importers::runtipi::read_seeds(
            std::path::Path::new(&path)
                .parent()
                .unwrap_or(std::path::Path::new(".")),
        )
    } else {
        (Vec::new(), Vec::new())
    };

    let facts = serde_json::json!({
        "id": id,
        "source": source,
        "services": template.plan.services.len(),
        "images": images,
        "published_container_port": published.map(|(_, port)| port.container),
        "preferred_host_port": published.map(|(_, port)| port.host),
        "generated_credentials": template.secrets.len(),
        "fields": fields,
        "managed_directories": managed,
        "shared_folders": shared,
        "named_volumes": volumes,
        "seed_files": seeds.iter().map(|seed| seed.path.as_str()).collect::<Vec<_>>(),
        "binary_seeds": binary_seeds,
        "limitations": outcome
            .limitations
            .iter()
            .map(|limit| {
                let mut row = BTreeMap::new();
                row.insert("feature", limit.feature().to_owned());
                row.insert("category", limit.category().to_owned());
                row
            })
            .collect::<Vec<_>>(),
    });
    println!("{}", serde_json::to_string_pretty(&facts).unwrap());
}
