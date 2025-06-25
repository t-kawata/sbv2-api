use std::fs;
use std::io::copy;
use std::path::Path;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let static_dir = home_dir().unwrap().join(".cache/sbv2");
    // let static_path = static_dir.join("all.bin");
    let dir_path = Path::new("../../../sbv2-dict/output");
    fs::create_dir_all(dir_path)?;
    let all_bin_path = dir_path.join("all.bin");
    let static_path = PathBuf::from(all_bin_path);
    // let out_path = PathBuf::from(&env::var("OUT_DIR").unwrap()).join("all.bin");
    println!("cargo:rerun-if-changed=build.rs");
    if static_path.exists() {
        println!("cargo:info=Dictionary file already exists, skipping download.");
    } else {
        println!("cargo:warning=Downloading dictionary file...");
        // let mut response =
        //     ureq::get("https://huggingface.co/neody/sbv2-api-assets/resolve/main/dic/all.bin")
        //         .call()?;
        let mut response =
            ureq::get("https://github.com/t-kawata/sbv2-dict/releases/download/v0.0.0/all.bin")
                .call()?;
        let mut response = response.body_mut().as_reader();
        // if !static_dir.exists() {
        //     fs::create_dir_all(static_dir)?;
        // }
        if let Some(parent) = static_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        let mut file = fs::File::create(&static_path)?;
        copy(&mut response, &mut file)?;
    }
    // if !out_path.exists() && fs::hard_link(&static_path, &out_path).is_err() {
    //     println!("cargo:warning=Failed to create hard link, copying instead.");
    //     fs::copy(static_path, out_path)?;
    // }
    Ok(())
}
