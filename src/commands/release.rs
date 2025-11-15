use crate::commands::config::ReleaseConfig;
use crate::commands::command_execution::{CommandExecutor, CommandParams};
use crate::commands::dmg;
use crate::commands::sync;
use crate::commands::version;
use anyhow::{Context, Result};
use std::env;

pub fn handle_release(dry_run: bool, major: bool, minor: bool, patch: bool, ver: Option<String>) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    // Check if we're in a directory with an .xcodeproj folder
    let xcodeproj_found = std::fs::read_dir(&current_dir)?
        .filter_map(|entry| entry.ok())
        .any(|entry| {
            entry.path().is_dir() &&
            entry.file_name().to_string_lossy().ends_with(".xcodeproj")
        });

    if !xcodeproj_found {
        eprintln!("Error: No .xcodeproj folder found in the current directory");
        std::process::exit(1);
    }

    // Check for .deployment file
    let deployment_file = current_dir.join(".deployment");
    if !deployment_file.exists() {
        eprintln!("Error: .deployment file not found in the current directory");
        eprintln!();
        eprintln!("You can create one based on the template:");
        eprintln!();
        print_deployment_template();
        std::process::exit(1);
    }

    // Parse deployment configuration from file
    let mut config = crate::commands::settings_parser::parse_deployment_file(deployment_file.to_str().unwrap(), dry_run)?;

    // Handle version incrementing if needed
    if major || minor || patch || ver.is_some() {
        // Get the current version from the project or config
        let current_version = if let Some(config_version) = &config.version {
            config_version.clone()
        } else {
            version::get_current_version(dry_run)?
        };

        let new_version = if let Some(explicit_version) = ver {
            explicit_version
        } else {
            version::increment_version(&current_version, major, minor, patch)?
        };

        // Update the version in config
        config.version = Some(new_version.clone());

        // Set the new version in the project using agvtool
        version::update_project_version(&new_version, dry_run)?;
    }

    // Get the version to be used for this release
    let version_to_release = if let Some(version) = &config.version {
        version.clone()
    } else {
        version::get_current_version(dry_run)?
    };

    // Check if this version already exists in the release folder
    let release_folder_path = current_dir.join(&config.release_folder);
    if version::check_version_exists(&release_folder_path, &version_to_release)? {
        eprintln!("Error: Version {} already exists in the release folder", version_to_release);
        eprintln!("  Path: {}", release_folder_path.join(&version_to_release).display());
        eprintln!();
        eprintln!("To release a new version, use one of the following flags:");
        eprintln!("  --major    Increment major version (X.0.0)");
        eprintln!("  --minor    Increment minor version (x.Y.0)");
        eprintln!("  --patch    Increment patch version (x.y.Z)");
        eprintln!("  --ver <version>   Set specific version");
        std::process::exit(1);
    }

    // Validate all required parameters are present
    crate::commands::settings_parser::validate_config(&config)?;

    // Execute the release process
    execute_release(&config)?;

    Ok(())
}

fn execute_release(config: &ReleaseConfig) -> Result<()> {
    let version = if let Some(version) = &config.version {
        version.clone()
    } else {
        // Get version from agvtool
        let executor = CommandExecutor::new(config.dry_run);
        let params = CommandParams::new("xcrun", &["agvtool", "what-version", "-terse"]);
        executor.execute_with_output(&params)?
    };

    if config.dry_run {
        println!("DRY RUN MODE - No actual commands will be executed");
        println!("Configuration loaded from .deployment file:");
        println!("  APP_NAME: {}", config.app_name);
        println!("  SCHEME: {}", config.scheme);
        println!("  EXPORT_METHOD: {}", config.export_method);
        println!("  ARCHIVE_FOLDER: {}", config.archive_folder);
        println!("  RELEASE_FOLDER: {}", config.release_folder);
        println!("Project version: {}", version);
        println!();
    }

    println!("Starting release process for version: {}", version);

    // Create the exportOptions.plist file
    create_export_options_plist(config, &version)?;

    // Step 1: Archive the current project state
    let archive_file = format!("{}/{}-{}.xcarchive", config.archive_folder, config.app_name, version);
    println!("Archiving project to: {}", archive_file);

    if !config.dry_run {
        std::fs::create_dir_all(&config.archive_folder)?;
    }

    let archive_params = CommandParams::new("xcodebuild", &[
        "archive",
        "-configuration", "Release",
        "-scheme", &config.scheme,
        "-archivePath", &archive_file,
    ]);
    let executor = CommandExecutor::new(config.dry_run);
    executor.execute(&archive_params)?;

    // Step 2: Export the app from the archive
    let release_version = format!("{}/{}", config.release_folder, version);
    println!("Exporting app to: {}", release_version);

    if !config.dry_run {
        std::fs::create_dir_all(&release_version)?;
    }

    let export_params = CommandParams::new("xcodebuild", &[
        "-exportArchive",
        "-archivePath", &archive_file,
        "-exportPath", &release_version,
        "-exportOptionsPlist", &config.export_options_plist,
    ]);
    let executor = CommandExecutor::new(config.dry_run);
    executor.execute(&export_params)?;

    // Step 3: Create a DMG file containing the app
    let app_path = format!("{}/{}.app", release_version, config.app_name);
    println!("Creating DMG for notarization...");

    // Use ZIP_NAME if provided, otherwise use app_name
    let dmg_name = config.zip_name.as_ref()
        .map(|name| name.trim_end_matches(".dmg").to_string())
        .unwrap_or_else(|| config.app_name.clone());

    let dmg_path = dmg::create_dmg(
        &app_path,
        &config.app_name,
        &version,
        &release_version,
        &dmg_name,
        config.dry_run,
    )?;

    // Step 4: Notarize the DMG with Apple
    println!("Submitting DMG for notarization...");

    let notarize_params = CommandParams::new("xcrun", &[
        "notarytool",
        "submit",
        &dmg_path,
        "--apple-id", &config.apple_id,
        "--team-id", &config.team_id,
        "--password", &config.account_token,
        "--wait"
    ]);
    let executor = CommandExecutor::new(config.dry_run);
    executor.execute(&notarize_params)?;

    // Step 5: Staple the notarization ticket to DMG
    println!("Stapling notarization ticket to DMG...");

    let staple_params = CommandParams::new("xcrun", &["stapler", "staple", &dmg_path]);
    let executor = CommandExecutor::new(config.dry_run);
    executor.execute(&staple_params)?;

    // Step 6: Copy stapled DMG to 'all' folder for appcast
    let all_release_folder = format!("{}/all", config.release_folder);
    if !config.dry_run {
        std::fs::create_dir_all(&all_release_folder)?;
    }

    // Copy the stapled DMG to the 'all' folder with version prefix
    // The DMG is named <dmg_name>.dmg, and in 'all' folder it becomes VERSION_<dmg_name>.dmg
    let version_dmg_name = format!("{}_{}.dmg", version, dmg_name);
    let appcast_dmg_path = format!("{}/{}", all_release_folder, version_dmg_name);
    println!("Copying stapled DMG to: {}", appcast_dmg_path);

    let executor = CommandExecutor::new(config.dry_run);
    let cp_dmg_params = CommandParams::new("cp", &[&dmg_path, &appcast_dmg_path]);
    executor.execute(&cp_dmg_params)?;

    // Generate the appcast in the 'all' folder
    println!("Generating appcast in: {}", all_release_folder);

    let appcast_params = CommandParams::new("generate_appcast", &[&all_release_folder]);
    let executor = CommandExecutor::new(config.dry_run);
    executor.execute(&appcast_params)?;

    // Step 7: Create sync folder and upload to remote server
    println!();
    sync::create_sync_folder(&config.release_folder, &config.sparkle_host, &dmg_name, config.dry_run)?;

    println!();
    println!("Release process completed for version: {}", version);
    Ok(())
}

fn create_export_options_plist(config: &ReleaseConfig, _version: &str) -> Result<()> {
    // For macOS app distribution where the build is already signed by Xcode,
    // we only need the export method and basic settings
    let export_options_content = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>method</key>
    <string>{}</string>
    <key>uploadBitcode</key>
    <true/>
    <key>uploadSymbols</key>
    <true/>
    <key>teamID</key>
    <string>{}</string>
</dict>
</plist>"#, config.export_method, config.team_id);

    // Write the export options plist to the specified path
    std::fs::write(&config.export_options_plist, export_options_content)?;

    Ok(())
}

fn print_deployment_template() {
    println!("{}", include_str!("../../template.deployment"));
}