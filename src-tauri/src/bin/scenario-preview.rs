//! Development-only bridge for inspecting the same Rust presentation in a browser.
fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let result = match arguments.as_slice() {
        [command, path, at]
            if command == "personal" || command == "import" || command == "mutate" =>
        {
            (|| {
                let path = std::path::Path::new(path);
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let mut store = temporal_app::store::Store::open(path)?;
                let now = at
                    .parse()
                    .map_err(|_| "Invalid injected preview time".to_string())?;
                if command == "import" {
                    use std::io::Read;
                    let mut json = String::new();
                    std::io::stdin()
                        .take((temporal_app::trace::MAX_BYTES + 1) as u64)
                        .read_to_string(&mut json)
                        .map_err(|e| e.to_string())?;
                    serde_json::to_string(&temporal_app::personal::import(&mut store, &json, now)?)
                        .map_err(|e| e.to_string())
                } else if command == "mutate" {
                    use std::io::Read;
                    let mut json = String::new();
                    std::io::stdin()
                        .take((temporal_app::trace::MAX_BYTES + 1) as u64)
                        .read_to_string(&mut json)
                        .map_err(|e| e.to_string())?;
                    let mutation: temporal_app::local::LocalMutation = serde_json::from_str(&json)
                        .map_err(|e| format!("Invalid local mutation: {e}"))?;
                    store.mutate_local(&mutation, now)?;
                    serde_json::to_string(&temporal_app::personal::view(&store, now)?)
                        .map_err(|e| e.to_string())
                } else {
                    serde_json::to_string(&temporal_app::personal::view(&store, now)?)
                        .map_err(|e| e.to_string())
                }
            })()
        }
        [catalog] if catalog == "catalog" => {
            serde_json::to_string(&temporal_app::catalog()).map_err(|e| e.to_string())
        }
        [id, offset] => offset
            .parse::<u32>()
            .map_err(|_| "invalid clock offset".into())
            .and_then(|offset| temporal_app::snapshot(id, offset))
            .and_then(|value| serde_json::to_string(&value).map_err(|e| e.to_string())),
        _ => Err("expected catalog or scenario ID and minute offset".into()),
    };
    match result {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
