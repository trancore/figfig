use anyhow::{bail, Context, Result};

#[cfg(target_os = "macos")]
pub async fn setup(remove: bool) -> Result<()> {
    use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, process::Command};
    let home = std::env::var_os("HOME").context("HOME is not set")?;
    let bundle = PathBuf::from(home).join("Library/Input Methods/figfig Cursor.app");
    let executable = bundle.join("Contents/MacOS/figfig-cursor");
    let plist = bundle.join("Contents/Info.plist");
    if bundle.exists() {
        let existing =
            fs::read_to_string(&plist).context("Inspect existing cursor tracker bundle")?;
        if !existing.contains("<string>dev.figfig.cursor</string>") {
            bail!(
                "Refusing to replace an unrelated input method at {}",
                bundle.display()
            );
        }
    }
    if remove {
        if bundle.exists() {
            crate::helper::ensure_running().await?;
            crate::helper::require_ack(
                crate::helper::exchange(figfig_core::ClientMessage::SetupCursor {
                    bundle_path: bundle.to_string_lossy().into(),
                    enabled: false,
                })
                .await?,
            )?;
            fs::remove_dir_all(&bundle)?;
        }
        println!("Removed figfig's shared integrated-terminal cursor tracker.");
        return Ok(());
    }
    fs::create_dir_all(executable.parent().unwrap())?;
    let temporary = executable.with_extension(format!("{}-tmp", std::process::id()));
    fs::write(
        &temporary,
        include_bytes!(concat!(env!("OUT_DIR"), "/figfig-cursor")),
    )?;
    fs::set_permissions(&temporary, fs::Permissions::from_mode(0o755))?;
    fs::rename(temporary, &executable)?;
    fs::write(
        &plist,
        include_str!("../../../macos/cursor/Info.plist")
            .replace("@VERSION@", env!("CARGO_PKG_VERSION")),
    )?;
    let resources = bundle.join("Contents/Resources");
    fs::create_dir_all(&resources)?;
    fs::write(
        resources.join("icon.icns"),
        include_bytes!("../../../src-tauri/icons/icon.icns"),
    )?;
    let status = Command::new("/usr/bin/codesign")
        .args(["--force", "--sign", "-"])
        .arg(&bundle)
        .status()
        .context("Sign the cursor tracker")?;
    if !status.success() {
        bail!("Could not sign the cursor tracker");
    }
    println!("Installed {}", bundle.display());
    println!("Registering figfig Cursor Tracking. A first installation may require logging out of macOS and back in before activation. It requests only supported integrated terminals' cursor geometry and does not handle keystrokes or change your keyboard layout.");
    crate::helper::ensure_running().await?;
    crate::helper::require_ack(
        crate::helper::exchange(figfig_core::ClientMessage::SetupCursor {
            bundle_path: bundle.to_string_lossy().into(),
            enabled: true,
        })
        .await?,
    )?;
    println!("Restart your editor and open a new zsh terminal. Run `figfig setup terminal --remove` to uninstall cursor tracking.");
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub async fn setup(_: bool) -> Result<()> {
    bail!("Integrated-terminal cursor tracking requires macOS");
}
