fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").ok().as_deref() != Some("macos") {
        return;
    }
    // screencapturekit emits this rpath only for its own artifacts. On macOS 11+
    // the Swift concurrency library lives in the dyld shared cache at this path
    // even when the file is not visible on disk. Do not also rpath the Command
    // Line Tools copy or both libraries load and ObjC classes collide.
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
}
