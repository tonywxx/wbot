//! Build script: locate and link the system TA-Lib (C) library.
//!
//! The wbot TA-Lib integration (`src/indicators/ta.rs`) talks to the real
//! TA-Lib via its C abstract API. We use `pkg-config` (which is already
//! configured on this machine) to discover the correct `-I`/`-L`/`-l` flags.
//! A Homebrew fallback path is used if `pkg-config` is unavailable.

use std::process::Command;

fn main() {
    // Homebrew prefix varies (Apple Silicon vs Intel). Probe a few well-known
    // locations so the fallback works on either architecture.
    let homebrew_roots = [
        "/opt/homebrew/opt/ta-lib",
        "/usr/local/opt/ta-lib",
        "/opt/homebrew/Cellar/ta-lib/0.7.1",
        "/usr/local/Cellar/ta-lib/0.7.1",
    ];

    if let Ok(out) = Command::new("pkg-config").args(["--libs", "--cflags", "ta-lib"]).output() {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            for tok in text.split_whitespace() {
                if let Some(lib) = tok.strip_prefix("-l") {
                    println!("cargo:rustc-link-lib={lib}");
                } else if let Some(dir) = tok.strip_prefix("-L") {
                    println!("cargo:rustc-link-search=native={dir}");
                }
                // `-I` (include path) is irrelevant for Rust FFI declarations
                // and is intentionally ignored here.
            }
            println!("cargo:rerun-if-changed=build.rs");
            return;
        }
    }

    // Fallback: try the known Homebrew locations.
    for root in homebrew_roots {
        let lib_dir = format!("{root}/lib");
        if std::path::Path::new(&format!("{lib_dir}/libta-lib.dylib")).exists()
            || std::path::Path::new(&format!("{lib_dir}/libta-lib.a")).exists()
        {
            println!("cargo:rustc-link-search=native={lib_dir}");
            println!("cargo:rustc-link-lib=ta-lib");
            println!("cargo:rerun-if-changed=build.rs");
            return;
        }
    }

    // Last resort: let the linker search the default system paths.
    println!("cargo:rustc-link-lib=ta-lib");
    println!("cargo:rerun-if-changed=build.rs");
}
