use std::{env, path::Path};

fn main() {
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let lib_paths = vec!["../zbar/.libs"];
    let static_libs = vec!["zbar"];
    let dynamic_libs = vec!["dbus-1", "jpeg", "X11", "xcb", "Xau", "Xdmcp", "systemd"];

    println!("cargo:rerun-if-changed=../zbar/.libs/libzbar.a");

    for path in lib_paths {
        println!(
            "cargo:rustc-link-search=native={}",
            Path::new(&root).join(path).display()
        );
    }

    for lib in static_libs {
        println!("cargo:rustc-link-lib=static={lib}");
    }

    for lib in dynamic_libs {
        println!("cargo:rustc-link-lib=dylib={lib}");
    }

    // TODO: Check if dbus is available when the dbus feature is enabled
}
