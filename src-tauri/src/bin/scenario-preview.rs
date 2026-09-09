//! Development-only bridge for inspecting the same Rust presentation in a browser.
fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let result = match arguments.as_slice() {
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
