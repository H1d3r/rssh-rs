use std::fs;

fn main() {
    let c_src_dir = "c_src";
    let mut c_files = Vec::new();

    // Read the c_src directory
    if let Ok(entries) = fs::read_dir(c_src_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                // Check if it's a file and has a .c extension
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "c" {
                            #[cfg(debug_assertions)]
                            println!("cargo:warning=Found C file: {}", path.display());

                            c_files.push(path);
                        }
                    }
                }
            }
        }
    } else {
        // Handle the case where c_src directory might not exist,
        // though in the temaplte it's expected.
        eprintln!(
            "cargo:warning=Directory {} not found or could not be read.",
            c_src_dir
        );
    }

    if !c_files.is_empty() {
        cc::Build::new()
            .files(c_files) // Use .files() for multiple files
            .compile("c_code"); // The name of the static library to be generated
    } else {
        // If no C files are found, print a warning.
        println!(
            "cargo:warning=No .c files found in {} directory. Skipping C code compilation.",
            c_src_dir
        );
    }

    // Ensure Cargo reruns the build script if any file in c_src changes.
    // This is important so that adding/removing/modifying C files triggers a recompile.
    println!("cargo:rerun-if-changed={}", c_src_dir);
}
