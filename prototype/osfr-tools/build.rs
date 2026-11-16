//! Give the shipped Windows programs a version resource (product name and version) so a
//! code-signing service can enforce them (SignPath's terms: "Set all product name attributes to
//! your project's name"; A50). Needs a resource compiler: `OSFR_WINDRES` (windres-compatible, e.g.
//! llvm-mingw's `x86_64-w64-mingw32-windres`) for the gnu targets, or `rc.exe` on PATH for msvc.
//! Without one a release build warns and goes on without the resource; other builds skip quietly.
//! `OSFR_RELEASE_VERSION` (the release tag without its "v") overrides the package version.
use std::{env, fs, path::PathBuf, process::Command};

const PRODUCT: &str = "Free Realms Expanded";
const SHIPPED: [(&str, &str); 3] = [
    ("osfr-gateway", "Free Realms Expanded realm gateway"),
    ("osfr-login", "Free Realms Expanded realm login"),
    ("osfr-account", "Free Realms Expanded account tool"),
];

fn main() {
    for var in ["OSFR_WINDRES", "OSFR_RELEASE_VERSION"] {
        println!("cargo:rerun-if-env-changed={var}");
    }
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let release = env::var("PROFILE").as_deref() == Ok("release");
    let msvc = env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");
    let windres = env::var("OSFR_WINDRES").ok();
    if !msvc && windres.is_none() {
        if release {
            println!("cargo:warning=OSFR_WINDRES is not set: the realm programs get no version resource");
        }
        return;
    }
    let version = env::var("OSFR_RELEASE_VERSION").unwrap_or_else(|_| env::var("CARGO_PKG_VERSION").unwrap());
    let numbers: Vec<u16> = version.split(['.', '-']).filter_map(|part| part.parse().ok()).collect();
    let number = |i: usize| numbers.get(i).copied().unwrap_or(0);
    let quad = format!("{},{},{},0", number(0), number(1), number(2));
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    for (bin, description) in SHIPPED {
        let script = out.join(format!("{bin}.rc"));
        let original = format!("{bin}.exe");
        let strings = [
            ("CompanyName", PRODUCT),
            ("FileDescription", description),
            ("FileVersion", version.as_str()),
            ("LegalCopyright", "GNU AGPL-3.0"),
            ("OriginalFilename", original.as_str()),
            ("ProductName", PRODUCT),
            ("ProductVersion", version.as_str()),
        ]
        .map(|(key, value)| format!("      VALUE \"{key}\", \"{value}\"\n"))
        .concat();
        let rc = format!(
            "1 VERSIONINFO\nFILEVERSION {quad}\nPRODUCTVERSION {quad}\nBEGIN\n  BLOCK \"StringFileInfo\"\n  BEGIN\n    BLOCK \"040904B0\"\n    BEGIN\n{strings}    END\n  END\n  BLOCK \"VarFileInfo\"\n  BEGIN\n    VALUE \"Translation\", 0x409, 1200\n  END\nEND\n"
        );
        fs::write(&script, rc).expect("write the version resource script");
        let (object, status) = match &windres {
            Some(windres) if !msvc => {
                let object = out.join(format!("{bin}.o"));
                let status = Command::new(windres).args(["-O", "coff", "-i"]).arg(&script).arg("-o").arg(&object).status();
                (object, status)
            }
            _ => {
                let object = out.join(format!("{bin}.res"));
                let status = Command::new("rc.exe").arg("/nologo").arg("/fo").arg(&object).arg(&script).status();
                (object, status)
            }
        };
        match status {
            Ok(status) if status.success() => println!("cargo:rustc-link-arg-bin={bin}={}", object.display()),
            other => println!("cargo:warning=the resource compiler failed for {bin} ({other:?}): no version resource"),
        }
    }
}
