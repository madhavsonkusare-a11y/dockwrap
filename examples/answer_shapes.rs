//! What the candidates that need answers are actually asking for.
fn main() {
    let root = std::path::PathBuf::from(".cache/definitions");
    let mut kinds: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for revision in std::fs::read_dir(&root).into_iter().flatten().flatten() {
        for entry in walk(&revision.path()) {
            let Ok(definition) = std::fs::read_to_string(entry.join("docker-compose.json")) else {
                continue;
            };
            let app = entry.file_name().unwrap().to_string_lossy().into_owned();
            let config = std::fs::read_to_string(entry.join("config.json")).ok();
            let Ok(outcome) =
                local_store::importers::runtipi::import(&app, &definition, config.as_deref())
            else {
                continue;
            };
            let Some(template) = outcome.template else {
                continue;
            };
            for field in template.fields.iter().filter(|f| f.required) {
                let kind = format!("{:?}", field.kind);
                let kind = kind.split(' ').next().unwrap_or("?").to_owned();
                kinds
                    .entry(format!("{kind} | {}", field.key))
                    .or_default()
                    .push(app.clone());
            }
        }
    }
    let mut rows: Vec<_> = kinds.into_iter().collect();
    rows.sort_by_key(|(_, apps)| std::cmp::Reverse(apps.len()));
    for (kind, apps) in rows.iter().take(24) {
        println!("{:3}  {kind}", apps.len());
    }
}

fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        for entry in std::fs::read_dir(&next).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.join("docker-compose.json").is_file() {
                    found.push(path.clone());
                }
                stack.push(path);
            }
        }
    }
    found
}
