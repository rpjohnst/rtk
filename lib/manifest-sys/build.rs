use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    Command::new("windres")
        .args(&["-O", "coff", "manifest.rc"])
        .arg(&format!("{}/manifest.res.o", out_dir))
        .status().unwrap();

    Command::new("ld")
        .args(&["-r", "--defsym", "rsrc=0", "-o", "manifest.o", "manifest.res.o"])
        .current_dir(&Path::new(&out_dir))
        .status().unwrap();

    Command::new("ar")
        .args(&["crus", "libmanifest.a", "manifest.o"])
        .current_dir(&Path::new(&out_dir))
        .status().unwrap();

    println!("cargo:rustc-flags=-L {} -l manifest:static", out_dir);
}
