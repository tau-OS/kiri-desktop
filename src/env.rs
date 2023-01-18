//! Environment module for d5
//! Loads environment configs from a group of files
// It should load files from .profile and .config
use color_eyre::Result;
use directories::{BaseDirs, UserDirs};
use std::{process::Command, path::PathBuf};
use tracing::{debug, log::warn};

pub fn load_envs(session: crate::cli::DisplayMode) -> Result<()> {
    let b = BaseDirs::new();
    let u = UserDirs::new();

    // set XDG desktop session type
    match session {
        crate::cli::DisplayMode::X11 => std::env::set_var("XDG_SESSION_TYPE", "x11"),
        crate::cli::DisplayMode::Wayland => std::env::set_var("XDG_SESSION_TYPE", "wayland"),
    }

    // set tracing target

    let mut envs: Vec<PathBuf> = vec![];

    if let Some(files) = b {
        // push as Path
        // source .profile specially because it's a bash script not a full env file
        tracing::span!(tracing::Level::DEBUG, "Loading .profile").in_scope({
            || {
                let profile_path = files.home_dir().join(".profile");
                Command::new("sh")
                    .arg("-c")
                    .arg(format!("source {}", profile_path.to_str().unwrap()))
                    .spawn().unwrap();
            }
        });

        // load from .config/environment.d
        // turns out systemd already takes care of this
        // for file in files
        //     .config_dir()
        //     .join("environment.d")
        //     .read_dir()?
        //     .filter(|f| f.is_ok())
        // {
        //     envs.push(file?.path());
        // }
    }

    for env in envs {
        let h =  dotenvy::from_path(&env);
        if h.is_ok() {
            debug!("Loaded env from: {:?}", env);
        } else {
            warn!("Failed to load env from {:?}: {:?}", env, h.err());
            // get error
        }
    }

    Ok(())
}

