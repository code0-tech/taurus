//! Exports every self-registered module (see `taurus_core::registry::build_modules`)
//! as a directory of JSON files, mirroring the old `definitions/*.json` layout.
//! Useful for eyeballing/diffing what Taurus would push to Aquila without a live
//! Aquila instance.
//!
//! Usage: `cargo run -p taurus-core --example export_definitions [output-dir]`
//! (defaults to `./export-definitions`).

fn main() {
    let output_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./export-definitions".to_string());
    let output_dir = std::path::Path::new(&output_dir);

    let modules = taurus_core::registry::build_modules();
    taurus_core::export::write_all(&modules, output_dir).expect("failed to export definitions");

    println!(
        "Exported {} module(s) to {}",
        modules.len(),
        output_dir.display()
    );
}
