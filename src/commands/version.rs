use crate::commands::command_execution::{CommandExecutor, CommandParams};
use anyhow::Result;
use std::path::Path;

/// Retrieves the current version from the Xcode project using agvtool
pub fn get_current_version(dry_run: bool) -> Result<String> {
    let executor = CommandExecutor::new(dry_run);
    let params = CommandParams::new("xcrun", &["agvtool", "what-version", "-terse"]);

    if dry_run {
        Ok("1.0.0".to_string()) // Mock value for dry run
    } else {
        executor.execute_with_output(&params)
    }
}

/// Increments a semantic version string based on the specified component
///
/// # Arguments
/// * `current_version` - Current version string in x.y.z format
/// * `major` - If true, increment major version and reset minor/patch to 0
/// * `minor` - If true, increment minor version and reset patch to 0
/// * `patch` - If true, increment patch version
pub fn increment_version(current_version: &str, major: bool, minor: bool, patch: bool) -> Result<String> {
    let parts: Vec<&str> = current_version.split('.').collect();
    if parts.len() < 3 {
        return Err(anyhow::anyhow!("Version format is not x.y.z: {}", current_version));
    }

    let major_part: u32 = parts[0].parse()
        .map_err(|_| anyhow::anyhow!("Invalid major version: {}", parts[0]))?;
    let minor_part: u32 = parts[1].parse()
        .map_err(|_| anyhow::anyhow!("Invalid minor version: {}", parts[1]))?;
    let patch_part: u32 = parts[2].parse()
        .map_err(|_| anyhow::anyhow!("Invalid patch version: {}", parts[2]))?;

    if major {
        let new_major = major_part + 1;
        Ok(format!("{}.0.0", new_major))
    } else if minor {
        let new_minor = minor_part + 1;
        Ok(format!("{}.{}.0", major_part, new_minor))
    } else if patch {
        let new_patch = patch_part + 1;
        Ok(format!("{}.{}.{}", major_part, minor_part, new_patch))
    } else {
        Ok(current_version.to_string())
    }
}

/// Checks if a version folder already exists in the release directory
///
/// # Arguments
/// * `release_folder` - Path to the release folder
/// * `version` - Version string to check
pub fn check_version_exists(release_folder: &Path, version: &str) -> Result<bool> {
    let version_folder = release_folder.join(version);
    Ok(version_folder.exists())
}

/// Updates the project version using agvtool
///
/// # Arguments
/// * `version` - New version string to set
/// * `dry_run` - If true, only print what would be done
pub fn update_project_version(version: &str, dry_run: bool) -> Result<()> {
    let executor = CommandExecutor::new(dry_run);
    let version_params = CommandParams::new("xcrun", &["agvtool", "new-version", "-all", version]);

    if !dry_run {
        println!("Updating project version to: {}", version);
        executor.execute(&version_params)?;
    } else {
        println!("DRY RUN: Would update project version to: {}", version);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment_major() {
        assert_eq!(increment_version("1.2.3", true, false, false).unwrap(), "2.0.0");
    }

    #[test]
    fn test_increment_minor() {
        assert_eq!(increment_version("1.2.3", false, true, false).unwrap(), "1.3.0");
    }

    #[test]
    fn test_increment_patch() {
        assert_eq!(increment_version("1.2.3", false, false, true).unwrap(), "1.2.4");
    }

    #[test]
    fn test_invalid_version_format() {
        assert!(increment_version("1.2", true, false, false).is_err());
    }
}
