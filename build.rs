use std::path::Path;

fn main() {
    let base = Path::new("leveldb");

    // Include paths for LevelDB headers
    let mut build = cc::Build::new();

    build
        .cpp(true)
        .flag_if_supported("-std=c++17")
        .warnings(false)
        .include(base.join("include"))
        .include(base.join("port"))
        .include(base.to_path_buf());

    // Compile LevelDB source files
    for dir in ["db", "table", "util", "port"] {
        let pattern = base.join(dir).join("*.cc");
        for entry in glob::glob(pattern.to_str().unwrap()).unwrap() {
            let path = entry.unwrap();
            let file_name = path.file_name().unwrap().to_string_lossy();
            // Skip certain files not needed for Bedrock
            // Skip test files
            if file_name.contains("_test")
                || file_name.contains("testutil")
                || file_name.contains("bench")
            {
                continue;
            }

            if cfg!(target_os = "windows") {
                // Skip files that are not compatible with Windows
                if file_name == "env_posix.cc" || file_name == "file_posix.cc" {
                    continue;
                }
            }

            if !cfg!(target_os = "windows") {
                // Skip Windows-specific files on non-Windows platforms
                if file_name == "env_win.cc" || file_name == "file_win.cc" {
                    continue;
                }
            }

            build.file(&path);
        }
    }

    if cfg!(target_os = "windows") {
        build.define("LEVELDB_PLATFORM_WINDOWS", None);
        build.define("NOMINMAX", None);
    } else if cfg!(target_os = "linux") {
        build.define("LEVELDB_PLATFORM_POSIX", None);
    } else if cfg!(target_os = "macos") {
        build.define("LEVELDB_PLATFORM_POSIX", None);
        build.flag_if_supported("-stdlib=libc++");
    }

    // Build as a static library
    build.compile("leveldb");

    // Link libraries needed by LevelDB
    println!("cargo:rustc-link-lib=static=zlib");
    println!("cargo:rustc-link-lib=snappy");
    println!(
        "cargo:rustc-link-search=native={}",
        "F:\\vcpkg\\installed\\x64-windows\\lib\\"
    );
    println!("cargo:rerun-if-changed=leveldb/");

    // Optional: print current submodule commit hash for debugging
    if let Ok(output) = std::process::Command::new("git")
        .args(&["-C", "leveldb", "rev-parse", "--short", "HEAD"])
        .output()
    {
        if let Ok(hash) = String::from_utf8(output.stdout) {
            println!("cargo:warning=Building LevelDB commit: {}", hash.trim());
        }
    }
}
