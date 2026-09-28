use std::process::Command;

const USER_BINS: &[(&str, u64)] = &[("hello_a", 0x8040_0000), ("hello_b", 0x8040_2000)];

fn main() {
    println!("cargo:rerun-if-changed=../user/src");
    println!("cargo:rerun-if-changed=../user/Cargo.toml");

    std::fs::create_dir_all("../target/user/scripts").unwrap();
    for (name, base) in USER_BINS {
        // 1. generate this bin's linker script at its own base
        let ld = format!("../target/user/scripts/{name}.ld");
        std::fs::write(&ld, linker_script(*base)).unwrap();
        let ld_abs = std::fs::canonicalize(&ld).unwrap();

        // 2. build it — isolated from kernel flags (same env_remove rule as before!)
        let status = Command::new("cargo")
            .args(["build", "--manifest-path", "../user/Cargo.toml",
                   "--bin", name, "--target", "riscv64gc-unknown-none-elf",
                   "--target-dir", "../target/user"])
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env_remove("RUSTFLAGS")
            .env("RUSTFLAGS", format!("-C link-arg=-T{} -C debuginfo=0", ld_abs.display()))
            .status().expect("failed to build user bin");
        assert!(status.success(), "user bin {name} build failed");

        // 3. ELF -> flat binary
        let elf = format!("../target/user/riscv64gc-unknown-none-elf/debug/{name}");
        let bin = format!("../target/user/{name}.bin");
        let st = Command::new(find_objcopy())
            .args(["--strip-all", "-O", "binary", &elf, &bin])
            .status().expect("no objcopy");
        assert!(st.success(), "objcopy {name} failed");

        // 4. hand bytes to rustc
        let out = std::env::var("OUT_DIR").unwrap();
        std::fs::copy(&bin, format!("{out}/{name}.bin")).unwrap();
    }
}

fn linker_script(base: u64) -> String {
    format!(r#"OUTPUT_ARCH(riscv)
ENTRY(_start)
SECTIONS {{
    . = {:#x};
    .text : {{
        KEEP(*(.text.entry))
        *(.text .text.*)
    }}
    .rodata : ALIGN(4) {{ *(.rodata .rodata.*) *(.srodata .srodata.*) }}
    .data : ALIGN(4) {{
        PROVIDE(__global_pointer$ = . + 0x800);
        *(.sdata .sdata.*) *(.data .data.*)
    }}
    .bss : ALIGN(4) {{ *(.sbss .sbss.*) *(.bss .bss.*) }}
    /DISCARD/ : {{ *(.eh_frame) *(.comment) *(.debug_*) }}
}}"#, base)
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
