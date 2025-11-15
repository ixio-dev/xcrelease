use crate::commands::command_execution::{CommandExecutor, CommandParams};
use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::Path;

/// Creates a sync folder structure and syncs to remote server
///
/// # Arguments
/// * `work_dir` - Working directory containing the 'all' folder
/// * `app_name` - Application name for remote path
/// * `dry_run` - Whether to run in dry-run mode
pub fn create_sync_folder(work_dir: &str, sparkle_path: &Option<String>, app_name: &str, dry_run: bool) -> Result<()> {
    let all_dir = format!("{}/all", work_dir);
    let sync_dir = format!("{}/sync", work_dir);

    // Validate directories
    if !dry_run {
        if !Path::new(work_dir).exists() {
            return Err(anyhow::anyhow!("WORK_DIR '{}' does not exist", work_dir));
        }

        if !Path::new(&all_dir).exists() {
            return Err(anyhow::anyhow!("'all' folder does not exist in '{}'", work_dir));
        }
    }

    println!("Creating sync folder structure...");

    // Create sync directory
    if !dry_run {
        fs::create_dir_all(&sync_dir)
            .context("Failed to create sync directory")?;
    } else {
        println!("  Command: mkdir -p {}", sync_dir);
    }

    // Process .dmg and .delta files
    if !dry_run {
        let entries = fs::read_dir(&all_dir)
            .context("Failed to read all directory")?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "dmg" || extension == "delta" {
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            process_file(&path, filename, &sync_dir)?;
                        }
                    }
                }
            }
        }
    } else {
        println!("  Processing *.dmg and *.delta files from {}", all_dir);
        println!("  - Files with underscores will be split into subdirectories");
        println!("  - Files without underscores will be copied to sync root");
    }

    // Process appcast.xml if it exists
    let source_appcast = format!("{}/appcast.xml", all_dir);
    let dest_appcast = format!("{}/appcast.xml", sync_dir);

    if !dry_run && Path::new(&source_appcast).exists() {
        process_appcast(&source_appcast, &dest_appcast)?;
    } else if dry_run {
        println!("  Processing appcast.xml (replacing _ with /)");
    }

    if sparkle_path.is_some() {
        // Rsync to remote server
        let release_path = sparkle_path.as_ref().unwrap();
        let executor = CommandExecutor::new(dry_run);
        let remote_path = format!("{}/{}", release_path, app_name);
        let sync_source = format!("{}/", sync_dir);

        println!("Syncing to remote server: {}", remote_path);
        let rsync_params = CommandParams::new("rsync", &[
            "-qchazP",
            &sync_source,
            &remote_path,
        ]);
        executor.execute(&rsync_params)?;
    } else {
        println!("Release not uploaded, no value for SPARKLE_PATH specified");
    }

    println!("Sync folder creation completed!");
    Ok(())
}

/// Process a single file - copy to sync directory with proper structure
fn process_file(source_path: &Path, filename: &str, sync_dir: &str) -> Result<()> {
    if filename.contains('_') {
        // Split on first underscore
        if let Some(underscore_pos) = filename.find('_') {
            let first_part = &filename[..underscore_pos];
            let second_part = &filename[underscore_pos + 1..];

            let subdir = format!("{}/{}", sync_dir, first_part);
            fs::create_dir_all(&subdir)
                .context("Failed to create subdirectory")?;

            let dest_path = format!("{}/{}", subdir, second_part);
            fs::copy(source_path, dest_path)
                .context("Failed to copy file")?;
        }
    } else {
        // Copy directly to sync folder
        let dest_path = format!("{}/{}", sync_dir, filename);
        fs::copy(source_path, dest_path)
            .context("Failed to copy file")?;
    }

    Ok(())
}

/// Process appcast.xml - replace underscores with forward slashes
fn process_appcast(source_path: &str, dest_path: &str) -> Result<()> {
    let content = fs::read_to_string(source_path)
        .context("Failed to read appcast.xml")?;

    // Replace underscores with forward slashes
    let re = Regex::new(r"_").unwrap();
    let transformed = re.replace_all(&content, "/");

    fs::write(dest_path, transformed.as_ref())
        .context("Failed to write appcast.xml")?;

    Ok(())
}
