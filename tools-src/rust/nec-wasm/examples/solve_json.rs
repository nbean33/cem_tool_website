//! Prints a command's JSON output natively, so it can be diffed against the wasm build.
//!
//! `cargo run -p nec-wasm --example solve_json -- <cmd> <args.json>`
fn main() {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().expect("command name");
    let path = args.next().expect("arguments file");
    let json = std::fs::read_to_string(path).expect("read arguments");
    println!("{}", nec_wasm::call(&cmd, &json, None).expect("command succeeds"));
}
