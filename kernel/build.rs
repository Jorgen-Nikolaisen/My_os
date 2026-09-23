use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../user/src/main.rs");
    println!("cargo:rerun-if-changed=../user/linker.ld");
    println!("cargo:rerun-if-changed=../user/Cargo.toml");

    let user_ld = format!(
        "{}/../user/linker.ld",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    );

    let status = Command::new("cargo")
        .args([
            "build",
            "--manifest-path", "../user/Cargo.toml",
            "--target", "riscv64gc-unknown-none-elf",
            "--target-dir", "../target/user",
        ])
        // CRITICAL: the parent cargo exports CARGO_ENCODED_RUSTFLAGS to 
        // The nested cargo inherits it, and it takes PRECEDENCE over the
        // RUSTFLAGS set below — so the user program would get linked with the
        // kernel's script: wrong base, wrong entry order
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTFLAGS")
        .env(
            "RUSTFLAGS",
            format!("-C link-arg=-T{} -C debuginfo=0", user_ld),
        )
        .status()
        .expect("failed to run cargo for user program");

    assert!(status.success(), "user program build failed");


    let user_elf = "../target/user/riscv64gc-unknown-none-elf/debug/user-program";
    let user_bin = "../target/user/user.bin";

    let status = Command::new(find_objcopy())
        .args(["--strip-all", "-O", "binary", user_elf, user_bin])
        .status()
        .expect("no objcopy available (sudo apt install binutils)");
    assert!(status.success(), "objcopy failed");


    let out_dir = std::env::var("OUT_DIR").unwrap();
    let bytes = std::fs::read(user_bin).expect("user.bin missing");
    std::fs::write(format!("{out_dir}/user.bin.o"), bytes).unwrap();
}

fn find_objcopy() -> String {
    // Prefer rustup's bundled llvm-objcopy; fall back to system binutils.
    if let Ok(home) = std::env::var("HOME") {
        let p = format!(
            "{}/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-objcopy",
            home
        );
        if std::path::Path::new(&p).exists() {
            return p;
        }
    }
    "objcopy".to_string()
}
