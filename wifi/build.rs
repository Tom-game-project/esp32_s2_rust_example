
fn main() {
    embuild::espidf::sysenv::output();
    let _ = dotenvy::from_filename(".env");

    for (key, value) in std::env::vars() {
        println!("cargo:rustc-env={}={}", key, value);
        //if key.starts_with("API_") || key.starts_with("APP_") { }
    }
}
