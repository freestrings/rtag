extern crate git2;

use std::env;
use std::fs;
use std::process::Command;
use std::path::Path;
use git2::Repository;

//
// - openssl 먼저 설치 되어있어야 한다. (https://github.com/sfackler/rust-openssl)
//
fn main() {
    let url = "https://github.com/larsbs/id3v2lib.git";

    let base_path_str = env::current_dir().unwrap();
    let base_path = Path::new(&base_path_str);
    let id3v2_path = base_path.join("target/debug/build/id3v2lib");
    let id3v2_build_path = id3v2_path.join("build");
    let id3v2_library_dir = id3v2_build_path.join("src");
    let id3v2_library_path = id3v2_library_dir.join("libid3v2.a");

    if id3v2_path.exists() {
        // TODO git pull
    } else {
        Repository::clone(url, id3v2_path).unwrap();
    }

    match fs::create_dir_all(id3v2_build_path.to_str().unwrap()) {
        Ok(()) => {
            let status = Command::new("cmake")
                .current_dir(id3v2_build_path.to_str().unwrap())
                .arg("..")
                .status()
                .expect("failed to cmake");
            assert!(status.success());

            let status = Command::new("make")
                .current_dir(id3v2_build_path.to_str().unwrap())
                .status()
                .expect("failed to make");
            assert!(status.success());

            assert!(id3v2_library_path.exists());

            println!("cargo:rustc-link-search=native={}", id3v2_library_dir.to_str().unwrap());
            println!("cargo:rustc-link-lib=static=id3v2");
        },
        Err(_) => ()
    }
}