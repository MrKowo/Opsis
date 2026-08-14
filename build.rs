use std::env;
use std::fs;
use std::path::Path;

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        println!("cargo:rerun-if-changed={}", src_path.display());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            let _ = fs::copy(&src_path, &dst_path);
        }
    }
    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=extensions");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let src_extensions = Path::new(&manifest_dir).join("extensions");

    if !src_extensions.exists() {
        return;
    }

    // Determine target directory: OUT_DIR is typically target/[<triple>/]<profile>/build/<pkg>-<hash>/out
    // Going up 3 directories gives target/[<triple>/]<profile>
    if let Ok(out_dir) = env::var("OUT_DIR") {
        if let Some(target_dir) = Path::new(&out_dir).ancestors().nth(3) {
            let dst_extensions = target_dir.join("extensions");
            if let Err(err) = copy_dir_all(&src_extensions, &dst_extensions) {
                eprintln!("[build.rs] Failed to copy extensions to target folder: {}", err);
            }
        }
    }
}
