// PyO3's `extension-module` feature should add the macOS undefined-symbol
// linker args automatically, but in some toolchains it does not. Emit them
// here unconditionally on macOS so a fresh checkout builds without setup.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-arg-cdylib=-undefined");
        println!("cargo:rustc-link-arg-cdylib=dynamic_lookup");
    }
}
