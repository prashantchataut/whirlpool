fn main() {
    println!("cargo:rerun-if-changed=../../assets/brand/muelle.ico");
    println!("cargo:rerun-if-changed=build.rs");
    let mut resources = winresource::WindowsResource::new();
    resources.set_icon("../../assets/brand/muelle.ico");
    resources.set("ProductName", "Muelle");
    resources.set("FileDescription", "Muelle settings");
    if let Err(error) = resources.compile() {
        // Cosmetic only: a missing resource compiler must not block the build.
        println!("cargo:warning=could not embed resources: {error}");
    }
}
