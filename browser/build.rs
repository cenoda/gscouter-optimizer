fn main() {
    println!("cargo:rustc-check-cfg=cfg(gso_browser)");
    println!("cargo:rustc-cfg=gso_browser");
}
