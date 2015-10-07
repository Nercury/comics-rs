use std::error::Error;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{ Path, PathBuf };
use std::collections::VecDeque;
use std::process;

fn main() {
    inner::gen_models("index_models");
    inner::gen_models("resizer_models");
    inner::gen_models("users_models");

    let out_dir = env::var_os("OUT_DIR").unwrap();

    let dest_path = Path::new(&out_dir).join("release.rs");
    let mut f = File::create(&dest_path).unwrap();

    let output = process::Command::new("git")
                     .arg("log")
                     .arg("--pretty=format:%h")
                     .arg("-n")
                     .arg("1")
                     .output()
                     .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
    let commit_num = output.stdout;

    let s = format!("
        pub fn version() -> &'static str {{
            \"{}\"
        }}
    ", String::from_utf8_lossy(&commit_num));

    f.write_all(&s.bytes().collect::<Vec<_>>()).unwrap();

    let src = Path::new("views").to_path_buf();
    let dst = Path::new("target");

    let mut copy_queue: VecDeque<PathBuf> = VecDeque::new();
    copy_queue.push_front(src);

    while let Some(dir) = copy_queue.pop_back() {
        let _ = fs::create_dir(dst.join(&dir));

        let list = fs::read_dir(dir.clone()).ok().expect(&format!("expected to read dir {:?}", dir));
        for maybe_entry in list {
            match maybe_entry {
                Ok(entry) => {
                    match fs::metadata(entry.path()) {
                        Ok(md) => if md.is_dir() {
                            copy_queue.push_front(entry.path().to_path_buf());
                        } else {
                            let _ = fs::copy(entry.path(), dst.join(entry.path()));
                        },
                        Err(why) => panic!("failed to get metadata for {:?}: {}", entry.path(), Error::description(&why)),
                    }
                },
                Err(why) => panic!("read entry in list: {}", Error::description(&why)),
            }
        }
    }
}

#[cfg(not(feature = "serde_macros"))]
mod inner {
    extern crate syntex;
    extern crate serde_codegen;

    use std::env;
    use std::path::Path;

    pub fn gen_models(file: &str) {
        let src_name = ["src/", file, ".rs.in"].concat();
        let dst_name = [file, ".rs"].concat();

        let out_dir = env::var_os("OUT_DIR").unwrap();

        let src = Path::new(&src_name);
        let dst = Path::new(&out_dir).join(&dst_name);

        let mut registry = syntex::Registry::new();

        serde_codegen::register(&mut registry);
        registry.expand("", &src, &dst).unwrap();
    }
}

#[cfg(feature = "serde_macros")]
mod inner {
    pub fn gen_models(_file: &str) { }
}
