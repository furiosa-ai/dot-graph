use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        if self.0.contains(name) {
            bindgen::callbacks::MacroParsingBehavior::Ignore
        } else {
            bindgen::callbacks::MacroParsingBehavior::Default
        }
    }
}

// https://fitzgeraldnick.com/2016/12/14/using-libbindgen-in-build-rs.html
fn main() {
    println!("cargo:rustc-link-lib=gvc");
    println!("cargo:rustc-link-lib=cgraph");

    // https://github.com/rust-lang/rust-bindgen/issues/687
    let ignored_macros = IgnoreMacros(
        vec![
            "FP_INFINITE".into(),
            "FP_NAN".into(),
            "FP_NORMAL".into(),
            "FP_SUBNORMAL".into(),
            "FP_ZERO".into(),
        ]
        .into_iter()
        .collect(),
    );

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(ignored_macros))
        .rustfmt_bindings(true)
        .generate() // Finish the builder and generate the bindings.
        .expect("unable to generate bindings");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs")).expect("cannot write bindings");
}
