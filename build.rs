fn main() {
    for path in [
        "packaging/windows/leaf.rc",
        "packaging/windows/leaf.ico",
        "packaging/windows/sequence.txt",
        "src/gnome/mod.rs",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let source = std::fs::read_to_string("src/gnome/mod.rs").unwrap();
    let version = source
        .split("const APP_VERSION: &str = \"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    let sequence = std::fs::read_to_string("packaging/windows/sequence.txt").unwrap();
    let resource = std::fs::read_to_string("packaging/windows/leaf.rc")
        .unwrap()
        .replace(
            "@LEAF_VERSION@",
            &format!("{version}-windows.{}", sequence.trim()),
        );
    let rc = output.join("leaf.rc");
    std::fs::write(&rc, resource).unwrap();
    let icon = output.join("leaf-icon.o");
    let status = std::process::Command::new("windres")
        .arg("-i")
        .arg(&rc)
        .args(["-O", "coff", "-o"])
        .arg(&icon)
        .status()
        .expect("Windows packaging requires MinGW windres");
    assert!(status.success(), "could not compile the Windows icon");
    println!(
        "cargo:rustc-link-arg-bin=wordcraft-gnome={}",
        icon.display()
    );
}
