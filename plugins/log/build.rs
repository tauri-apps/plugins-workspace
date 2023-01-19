fn alias(alias: &str, has_feature: bool) {
    if has_feature {
        println!("cargo:rustc-cfg={alias}");
    }
}

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let mobile = target_os == "ios" || target_os == "android";
    alias("desktop", !mobile);
    alias("mobile", mobile);
}
