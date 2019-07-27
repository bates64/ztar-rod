use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=app");

    // Build TypeScript app.
    if !Command::new("npm")
        .args(&["run", "build"])
        .status()
        .unwrap()
        .success()
    {
        panic!("typescript build failed");
    }
}
