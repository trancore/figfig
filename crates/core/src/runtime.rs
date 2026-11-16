use crate::Settings;
use anyhow::{bail, Context, Result};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt},
    path::PathBuf,
};

pub fn runtime_dir() -> Result<PathBuf> {
    let uid = unsafe { getuid() };
    let path = std::env::var_os("FIGFIG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join(format!("figfig-{uid}")));
    match fs::DirBuilder::new().mode(0o700).create(&path) {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
        Err(e) => return Err(e.into()),
    }
    let metadata = fs::symlink_metadata(&path)?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != uid
        || metadata.permissions().mode() & 0o077 != 0
    {
        bail!(
            "Insecure runtime directory: {} (must be owned by you with mode 0700)",
            path.display()
        );
    }
    Ok(path)
}

unsafe extern "C" {
    fn getuid() -> u32;
}

pub fn socket_path() -> Result<PathBuf> {
    Ok(runtime_dir()?.join("app.sock"))
}

pub fn settings_path() -> PathBuf {
    std::env::var_os("FIGFIG_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                .join("Library/Application Support/figfig")
        })
        .join("settings.json")
}

pub fn load_settings() -> Result<Settings> {
    let path = settings_path();
    match fs::read(&path) {
        Ok(bytes) => Ok(serde_json::from_slice::<Settings>(&bytes)
            .with_context(|| format!("Invalid settings: {}", path.display()))?
            .normalize()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(e.into()),
    }
}

pub fn save_settings(settings: Settings) -> Result<Settings> {
    let settings = settings.normalize();
    let path = settings_path();
    let dir = path.parent().context("Missing settings directory")?;
    fs::create_dir_all(dir)?;
    let temp = dir.join(format!("settings-{}.tmp", std::process::id()));
    fs::write(&temp, serde_json::to_vec_pretty(&settings)?)?;
    fs::rename(temp, path)?;
    Ok(settings)
}
