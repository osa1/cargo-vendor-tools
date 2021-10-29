use std::path::{Path, PathBuf};

use clap::{App, Arg};
use serde_json as json;
use sha2::{Digest, Sha256};

fn main() {
    let args = App::new("cargo-compute-checksums")
        .arg(
            Arg::with_name("dir")
                .value_name("DIR")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("tarball_checksum")
                .value_name("TARBALL_CHECKSUM")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let dir = args.value_of("dir").unwrap();
    let tarball_checksum = args.value_of("tarball_checksum").unwrap();

    let mut file_checksums: json::map::Map<String, json::Value> = Default::default();

    generate_file_checksums(dir, PathBuf::from(dir).as_ref(), &mut file_checksums);

    let mut json_map = json::map::Map::new();
    json_map.insert("files".to_owned(), json::Value::Object(file_checksums));
    json_map.insert(
        "package".to_owned(),
        json::Value::String(tarball_checksum.to_owned()),
    );

    json::ser::to_writer_pretty(std::io::stdout(), &json_map).unwrap();
}

static IGNORED_FILES: [&str; 7] = [
    ".",
    "..",
    ".gitattributes",
    ".gitignore",
    ".cargo-ok",
    ".cargo-checksum.json",
    ".cargo_vcs_info.json",
];

fn generate_file_checksums(
    root: &str,
    dir: &Path,
    file_checksums: &mut json::map::Map<String, json::Value>,
) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();

        if IGNORED_FILES
            .iter()
            .any(|ignored_file| *ignored_file == file_name)
        {
            continue;
        }

        match entry.file_type() {
            Err(err) => {
                eprintln!(
                    "Error while getting type of file {:?}: {:?}",
                    file_name, err
                );
            }
            Ok(file_type) => {
                if file_type.is_dir() {
                    let mut dir_path = PathBuf::new();
                    dir_path.push(dir);
                    dir_path.push(entry.file_name());
                    generate_file_checksums(root, &dir_path, file_checksums);
                } else {
                    let mut file_path = PathBuf::new();
                    file_path.push(dir);
                    file_path.push(entry.file_name());

                    let rel_file_path = file_path.strip_prefix(root).unwrap();

                    let file_contents = std::fs::read(&file_path).unwrap();

                    let mut hasher = Sha256::new();
                    hasher.update(&file_contents);
                    let hash = hasher.finalize();
                    let hash_str = format!("{:x}", hash);

                    let old = file_checksums.insert(
                        rel_file_path.to_string_lossy().to_string(),
                        json::Value::String(hash_str),
                    );
                    assert!(old.is_none());
                }
            }
        }
    }
}
