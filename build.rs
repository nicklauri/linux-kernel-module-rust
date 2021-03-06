extern crate bindgen;
extern crate cc;
extern crate shlex;

use std::env;
use std::path::PathBuf;
use std::process::Command;

const INCLUDED_TYPES: &[&str] = &["file_system_type", "mode_t", "umode_t", "ctl_table"];
const INCLUDED_FUNCTIONS: &[&str] = &[
    "register_filesystem",
    "unregister_filesystem",
    "krealloc",
    "kfree",
    "mount_nodev",
    "kill_litter_super",
    "register_sysctl",
    "unregister_sysctl_table",
    "access_ok",
    "_copy_to_user",
    "_copy_from_user",
];
const INCLUDED_VARS: &[&str] = &[
    "EINVAL",
    "ENOMEM",
    "EFAULT",
    "__this_module",
    "FS_REQUIRES_DEV",
    "FS_BINARY_MOUNTDATA",
    "FS_HAS_SUBTYPE",
    "FS_USERNS_MOUNT",
    "FS_RENAME_DOES_D_MOVE",
    "BINDINGS_GFP_KERNEL",
    "KERN_INFO",
    "VERIFY_WRITE",
];

fn main() {
    let mut builder = bindgen::Builder::default()
        .use_core()
        .ctypes_prefix("c_types")
        .no_copy(".*")
        .derive_default(true)
        .rustfmt_bindings(true);

    let output = String::from_utf8(
        Command::new("make")
            .arg("-C")
            .arg("kernel-cflags-finder")
            .arg("-s")
            .output()
            .unwrap()
            .stdout,
    ).unwrap();

    for arg in shlex::split(&output).unwrap() {
        builder = builder.clang_arg(arg.to_string());
    }

    println!("cargo:rerun-if-changed=src/bindings_helper.h");
    builder = builder.header("src/bindings_helper.h");

    for t in INCLUDED_TYPES {
        builder = builder.whitelist_type(t);
    }
    for f in INCLUDED_FUNCTIONS {
        builder = builder.whitelist_function(f);
    }
    for v in INCLUDED_VARS {
        builder = builder.whitelist_var(v);
    }
    let bindings = builder.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    let mut builder = cc::Build::new();
    println!("cargo:rerun-if-env-changed=CLANG");
    builder.compiler(env::var("CLANG").unwrap_or("clang".to_string()));
    builder.warnings(false);
    builder.file("src/helpers.c");
    for arg in shlex::split(&output).unwrap() {
        builder.flag(&arg);
    }
    builder.compile("helpers");
}
