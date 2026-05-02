use api_openapi::apidoc::full_spec;

fn main() {
    let spec = full_spec();
    println!(
        "{}",
        serde_json::to_string_pretty(&spec).expect("serialize spec")
    );
}
