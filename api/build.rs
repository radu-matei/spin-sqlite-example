fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let lib_dir = format!("{manifest_dir}/../libsqlite/lib/wasm32-wasi");
    println!("cargo:rustc-link-search=native={lib_dir}");
}
