fn main() {
    #[cfg(feature = "static")]
    println!("cargo:rustc-link-arg=-nostartfiles");
}
