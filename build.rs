fn main() {
    println!(
        "cargo:rustc-env=ENUMTRAIT_OUT_DIR={}",
        std::env::var("OUT_DIR").unwrap()
    )
}