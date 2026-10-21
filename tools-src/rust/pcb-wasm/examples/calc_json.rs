//! Runs `calculate` natively over a JSON list of `{ id, params }`, one result per line, so the
//! output can be diffed against the WebAssembly build.
fn main() {
    let path = std::env::args().nth(1).expect("cases file");
    let cases: Vec<serde_json::Value> = serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("json");
    for c in cases {
        let id = c["id"].as_str().expect("id");
        let out = pcb_wasm::calculate(id, &c["params"].to_string());
        println!("{}\t{}", id, out.unwrap_or_else(|e| format!("ERROR {e}")));
    }
}
