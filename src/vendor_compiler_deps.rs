use std::io::Write;
use std::process::{Command, Output, Stdio};

use clap::{App, Arg};

static DESCR: &str = "
TODO
";

fn main() {
    let args = App::new("vendor-compiler-deps")
        .long_about(DESCR)
        .arg(
            Arg::with_name("rust_install_path")
                .value_name("RUST_INSTALL_PATH")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("vendor_dir")
                .value_name("VENDOR_DIR")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let rust_install_path = args.value_of("rust_install_path").unwrap();
    let vendor_dir = args.value_of("vendor_dir").unwrap();

    let compiler_cargo_lock_path = format!(
        "{}/{}",
        rust_install_path, "lib/rustlib/src/rust/Cargo.lock"
    );

    println!("Reading compiler Cargo.lock ...");
    let compiler_deps = cargo_lock_deps(&compiler_cargo_lock_path);

    println!(
        "Fetching and unpacking compiler dependencies in {} ...",
        vendor_dir
    );
    for Dep { url, name, tarball_checksum } in compiler_deps {
        println!("  {}: {}", name, url);

        //
        // Fetch the package tarball
        //

        // curl --location <url>
        let Output { status, stdout: curl_stdout, stderr } = Command::new("curl")
            .args(["--location", &url])
            .output()
            .unwrap();

        if !status.success() {
            panic!(
                "`curl --location {}` returned {:?}:\n{}",
                url,
                status.code(),
                String::from_utf8_lossy(&stderr)
            );
        }

        let package_vendor_dir = format!("{}/{}", vendor_dir, name);

        //
        // Create package vendor dir
        //

        std::fs::create_dir_all(&package_vendor_dir).unwrap();

        //
        // Unpack package
        //

        let mut tar_cmd = Command::new("tar")
            .args(["xz"])
            .current_dir(&package_vendor_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let mut tar_stdin = tar_cmd.stdin.take().unwrap();
        tar_stdin.write_all(&curl_stdout).unwrap();
        drop(tar_stdin);

        let Output { status, stdout, stderr } = tar_cmd.wait_with_output().unwrap();

        if !status.success() {
            panic!(
                "`tar xz` returned {:?}:\n{}",
                status.code(),
                String::from_utf8_lossy(&stderr)
            );
        }
    }
}

#[derive(Debug)]
struct Dep {
    /// URL to fetch the package tarball
    url: String,

    /// <package-name>-<version>
    name: String,

    /// Expected checksum (SHA256) of the tarball
    tarball_checksum: String,
}

fn cargo_lock_deps(lock_file_path: &str) -> Vec<Dep> {
    let mut deps = vec![];

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
                deps.push(Dep {
                    url: format!(
                        "https://crates.io/api/v1/crates/{}/{}/download",
                        name, version
                    ),
                    name: format!("{}-{}", name, version),
                    tarball_checksum: checksum.to_owned(),
                });
            }
        }
    }

    deps
}
