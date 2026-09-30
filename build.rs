fn main() {
    // Future: embed icon.ico via winres
    // For now, just print cargo directives
    println!("cargo:rerun-if-changed=build.rs");
}
