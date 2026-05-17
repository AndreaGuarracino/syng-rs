//! Build script for syng-rs.
//!
//! Compiles the vendored syng C sources (`vendor/syng/*.c`) into a static
//! library `libsyng_rs.a` that the Rust FFI bindings in `src/lib.rs` link
//! against.
//!
//! Mirrors the syng project's own `vendor/syng/Makefile`: O3, no special
//! defines except `ONEIO` for the C files that use ONElib I/O.

use std::path::Path;

fn main() {
    let syng_dir = Path::new("vendor/syng");

    // Rebuild when any of the syng C/H sources change.
    for entry in std::fs::read_dir(syng_dir).expect("vendor/syng missing — did you `git submodule update --init`?") {
        let entry = entry.expect("read_dir entry failed");
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "c" || ext == "h" {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }

    // C files compiled into libsyng_rs.a.
    // Mirrors the upstream syng Makefile's compile units, omitting the
    // standalone-binary entry points (syng.c, syngmap.c, syngpath2gbwt.c,
    // syngstat.c, k31type.c, ONEview.c) which contain `main`.
    let c_files_with_oneio = [
        "seqio.c",      // -DONEIO
        "kmerhash.c",   // -DONEIO
    ];
    let c_files_plain = [
        "syngbwt3.c",
        "rskip.c",
        "syncmerset.c",
        "seqhash.c",
        "ONElib.c",
        "hash.c",
        "dict.c",
        "array.c",
        "utils.c",
    ];

    let mut build = cc::Build::new();
    build
        .include(syng_dir)
        .opt_level(3)
        .flag_if_supported("-fPIC")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-variable")
        .flag_if_supported("-Wno-unused-but-set-variable")
        .flag_if_supported("-Wno-sign-compare")
        .flag_if_supported("-Wno-implicit-fallthrough")
        .flag_if_supported("-Wno-format-truncation")
        .flag_if_supported("-Wno-stringop-truncation")
        .flag_if_supported("-Wno-stringop-overflow")
        .warnings(false);

    for f in &c_files_plain {
        build.file(syng_dir.join(f));
    }
    // Our own helper that re-exports static globals from syng.h.
    println!("cargo:rerun-if-changed=src/syng_helpers.c");
    build.file("src/syng_helpers.c").define("ONEIO", None);

    for f in &c_files_with_oneio {
        let mut b = build.clone();
        b.define("ONEIO", None).file(syng_dir.join(f)).compile(&format!("syng_{}", f.trim_end_matches(".c")));
    }
    build.compile("syng_core");

    // Link system libs syng depends on.
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=m");
}
