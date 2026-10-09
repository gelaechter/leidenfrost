fn main() {
    // Load the DATABASE_URL from the environment so SQLx can do compile time checking
    println!("cargo:rerun-if-env-changed=DATABASE_URL");

    let url = std::env::var("DATABASE_URL")
        .unwrap_or("sqlite://backend/src/db/test_data.sqlite".to_owned());

    println!("cargo:rustc-env=DATABASE_URL={url}");
}
