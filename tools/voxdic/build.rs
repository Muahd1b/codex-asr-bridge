use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=ASR_VOXTRAL_ROOT");

    let voxtral_root = resolve_voxtral_root();
    if !voxtral_root.exists() {
        panic!(
            "Voxtral source root not found: {} (set ASR_VOXTRAL_ROOT)",
            voxtral_root.display()
        );
    }

    let sources = [
        "voxtral.c",
        "voxtral_kernels.c",
        "voxtral_audio.c",
        "voxtral_encoder.c",
        "voxtral_decoder.c",
        "voxtral_tokenizer.c",
        "voxtral_safetensors.c",
        "voxtral_metal.m",
    ];

    for src in &sources {
        println!(
            "cargo:rerun-if-changed={}",
            voxtral_root.join(src).display()
        );
    }
    println!(
        "cargo:rerun-if-changed={}",
        voxtral_root.join("voxtral.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        voxtral_root.join("voxtral_metal.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        voxtral_root.join("voxtral_shaders_source.h").display()
    );

    let mut build = cc::Build::new();
    build
        .include(&voxtral_root)
        .define("USE_BLAS", None)
        .define("USE_METAL", None)
        .define("ACCELERATE_NEW_LAPACK", None)
        .flag("-O3")
        .flag("-ffast-math")
        .flag("-fobjc-arc")
        .warnings(true);

    for src in &sources {
        build.file(voxtral_root.join(src));
    }
    build.compile("voxtral_embed");

    println!("cargo:rustc-link-lib=framework=Accelerate");
    println!("cargo:rustc-link-lib=framework=Metal");
    println!("cargo:rustc-link-lib=framework=MetalPerformanceShaders");
    println!("cargo:rustc-link-lib=framework=MetalPerformanceShadersGraph");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=AudioToolbox");
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    println!("cargo:rustc-link-lib=objc");
}

fn resolve_voxtral_root() -> PathBuf {
    if let Ok(v) = env::var("ASR_VOXTRAL_ROOT") {
        return PathBuf::from(v);
    }
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join("DEV/voxtral.c");
    }
    PathBuf::from("/tmp/voxtral.c")
}
