fn main() {
    println!("cargo:rustc-link-lib=shapes_interfaces__rosidl_typesupport_c");
    println!("cargo:rustc-link-lib=shapes_interfaces__rosidl_generator_c");

    if let Some(e) = std::env::var_os("AMENT_PREFIX_PATH") {
        let env = e.to_str().unwrap();
        for path in env.split(':') {
            println!("cargo:rustc-link-search={path}/lib");
        }
    }
}
