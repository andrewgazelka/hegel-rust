use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PYTHON_VERSION: &str = "3.13";

// In order:
// * Prefer `hegel` on PATH
// * If not found, install hegel with uv
//    * Prefer `uv` on PATH
//    * If not found, install uv from installer
//
// All artifacts are installed to `OUT_DIR / hegel`.
//
// HEGEL_BINARY_PATH is exported for use by the code.

fn main() {
    // make our installed uv work under nix + madness:
    // https://github.com/antithesishq/madness
    //
    // note that this is now the default in more recent madness
    // versions, so we can eventually remove this
    std::env::set_var("MADNESS_ALLOW_LDD", "1");

    let hegel_path = ensure_hegel();
    eprintln!("using hegel: {}", hegel_path.display());
    // export HEGEL_BINARY_PATH for use by our code
    println!("cargo:rustc-env=HEGEL_BINARY_PATH={}", hegel_path.display());
}

fn ensure_hegel() -> PathBuf {
    if let Some(path) = find_on_path("hegel") {
        eprintln!("found hegel on path: {}", path.display());
        return path;
    }

    let install_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("hegel");
    let python_install_dir = install_path.join("python");

    fs::create_dir_all(&install_path)
        .unwrap_or_else(|_| panic!("failed to create {}", install_path.display()));

    let uv_path = ensure_uv(&install_path);
    eprintln!("using uv: {}", uv_path.display());

    let python_path = ensure_python(&uv_path, &python_install_dir);
    eprintln!("using python: {}", python_path.display());

    let python_bin_dir = python_path.parent().unwrap();
    let hegel_path = python_bin_dir.join("hegel");

    if hegel_path.exists() {
        eprintln!("found hegel: {}", hegel_path.display());
        return hegel_path;
    }

    eprintln!("installing hegel");
    let status = Command::new(&uv_path)
        .args([
            "pip",
            "install",
            // We install directly into our isolated Python rather than creating a venv.
            // Python uses /proc/self/exe to detect if it's running in a venv, but on
            // nix+madness systems this points to the dynamic linker instead of the Python
            // binary. Without venv detection, sys.prefix isn't set correctly and packages
            // installed in the venv aren't importable.
            // --break-system-packages bypasses the EXTERNALLY-MANAGED check.
            "--break-system-packages",
            "git+ssh://git@github.com/antithesishq/hegel.git",
            "--python",
        ])
        .arg(&python_path)
        .status()
        .expect("failed to install hegel");
    assert!(status.success(), "failed to install hegel");
    assert!(
        hegel_path.exists(),
        "hegel not found after installation: {}",
        hegel_path.display()
    );

    hegel_path
}

fn find_python(uv_path: &PathBuf, python_install_dir: &PathBuf) -> Option<PathBuf> {
    // UV_PYTHON_INSTALL_DIR restricts the search to our local dir, and
    // --managed-python prevents falling back to the system python
    let output = Command::new(uv_path)
        .args(["python", "find", PYTHON_VERSION, "--managed-python"])
        .env("UV_PYTHON_INSTALL_DIR", python_install_dir)
        .output()
        .expect("failed to run uv python find");

    if output.status.success() {
        let python_path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
        if python_path.exists() {
            assert!(
                python_path.starts_with(python_install_dir),
                "found python at {} but expected it to be in {}",
                python_path.display(),
                python_install_dir.display()
            );
            return Some(python_path);
        }
    }
    None
}

fn ensure_python(uv_path: &PathBuf, python_install_dir: &PathBuf) -> PathBuf {
    if let Some(python_path) = find_python(uv_path, python_install_dir) {
        eprintln!("found python: {}", python_path.display());
        return python_path;
    }

    // install a fresh copy of python to our local directory.
    // --no-bin prevents uv from creating symlinks in ~/.local/bin.
    eprintln!(
        "installing python {} at {}",
        PYTHON_VERSION,
        python_install_dir.display()
    );
    let status = Command::new(uv_path)
        .args(["python", "install", PYTHON_VERSION, "--install-dir"])
        .arg(python_install_dir)
        .arg("--no-bin")
        .status()
        .expect("failed to install python");
    assert!(status.success(), "failed to install python");

    find_python(uv_path, python_install_dir).expect("failed to find python after install")
}

fn ensure_uv(install_path: &Path) -> PathBuf {
    if let Some(path) = find_on_path("uv") {
        eprintln!("found uv on PATH: {}", path.display());
        return path;
    }

    let uv_path = install_path.join("uv");
    if uv_path.exists() {
        eprintln!("found uv: {}", uv_path.display());
        return uv_path;
    }

    eprintln!("installing uv");
    let status = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "curl -LsSf https://astral.sh/uv/install.sh | UV_INSTALL_DIR={} UV_NO_MODIFY_PATH=1 sh",
            install_path.display()
        ))
        .status()
        .expect("uv install script failed");
    assert!(status.success(), "uv install script failed");
    assert!(
        uv_path.exists(),
        "uv not found at {} after installation",
        uv_path.display()
    );

    uv_path
}

fn find_on_path(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join(name);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    })
}
