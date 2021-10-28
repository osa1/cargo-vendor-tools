use clap::{App, Arg};

fn main() {
    let args = App::new("cargo-list-lock")
        .arg(
            Arg::with_name("lock_file_path")
                .value_name("LOCK_FILE_PATH")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let lock_file_path = args.value_of("lock_file_path").unwrap();

    let lock_file = std::fs::read_to_string(lock_file_path).unwrap();

    let toml_value: toml::value::Value = toml::de::from_str(&lock_file).unwrap();

    let packages = toml_value.as_table().unwrap().get("package").unwrap();

    for package in packages.as_array().unwrap() {
        let package_tbl = package.as_table().unwrap();
        if let Some(source) = package_tbl.get("source") {
            if source.as_str().unwrap() == "registry+https://github.com/rust-lang/crates.io-index" {
                let name = package_tbl.get("name").unwrap().as_str().unwrap();
                let version = package_tbl.get("version").unwrap().as_str().unwrap();
                let checksum = package_tbl.get("checksum").unwrap().as_str().unwrap();
                println!(
                    "https://crates.io/api/v1/crates/{}/{}/download {}-{} {}",
                    name, version, name, version, checksum
                );
            }
        }
    }
}
