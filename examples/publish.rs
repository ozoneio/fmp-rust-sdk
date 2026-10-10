use std::{io, path::Path, process::Command};

/// Publishes the current crate using CARGO_REGISTRY_TOKEN from an explicit .env
/// file. The token is passed only to Cargo's child process, never to its arguments
/// or the parent process environment. Run after the release checks pass.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    let entries = dotenvy::from_path_iter(&path).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "cannot read .env for publishing credentials",
        )
    })?;
    let mut token: Option<String> = None;
    for entry in entries {
        let (name, value) = entry.map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid .env syntax; check quoting and assignments",
            )
        })?;
        if name == "CARGO_REGISTRY_TOKEN" {
            if token.is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "CARGO_REGISTRY_TOKEN appears more than once in .env",
                )
                .into());
            }
            token = Some(value);
        }
    }
    let token = token
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "set CARGO_REGISTRY_TOKEN in .env to a crates.io token with publishing permission",
            )
        })?;
    let status = Command::new("cargo")
        .args(["publish", "--locked"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("CARGO_REGISTRY_TOKEN", token)
        .status()?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "cargo publish failed with {status}; review Cargo's diagnostics above"
        ))
        .into());
    }
    Ok(())
}
