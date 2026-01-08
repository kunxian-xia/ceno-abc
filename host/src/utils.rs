use cargo_metadata::MetadataCommand;
use std::env;
use std::path::{Path, PathBuf};

pub(crate) fn discover_workspace_root() -> PathBuf {
    if let Ok(path) = env::var("WORKSPACE_ROOT") {
        let pb = PathBuf::from(path);
        eprintln!("WORKSPACE_ROOT (env) = {}", pb.display());
        return pb;
    }

    if let Ok(metadata) = MetadataCommand::new().no_deps().exec() {
        let root = metadata.workspace_root.into_std_path_buf();
        eprintln!("WORKSPACE_ROOT (cargo-metadata) = {}", root.display());
        return root;
    }

    if let Ok(exe_path) = env::current_exe() {
        let mut dir = exe_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        loop {
            if dir.join("Cargo.lock").exists() {
                eprintln!("WORKSPACE_ROOT (inferred from exe) = {}", dir.display());
                return dir;
            }
            if !dir.pop() {
                break;
            }
        }
    }

    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    eprintln!("WORKSPACE_ROOT fallback to cwd = {}", cwd.display());
    cwd
}
