use crate::commands::command_execution::{CommandExecutor, CommandParams};
use anyhow::{Context, Result};
use std::path::Path;

/// Creates a DMG file from an app bundle
///
/// # Arguments
/// * `app_path` - Path to the .app bundle
/// * `app_name` - Name of the application (without .app extension)
/// * `version` - Version string for the DMG
/// * `output_dir` - Directory where the DMG will be created
/// * `dmg_name` - Name for the DMG file (without .dmg extension)
/// * `dry_run` - Whether to run in dry-run mode
///
/// # Returns
/// Path to the created DMG file
pub fn create_dmg(
    app_path: &str,
    app_name: &str,
    version: &str,
    output_dir: &str,
    dmg_name: &str,
    dry_run: bool,
) -> Result<String> {
    let executor = CommandExecutor::new(dry_run);

    let volume_name = format!("{} {}", app_name, version);
    let temp_dmg = format!("temp-{}.dmg", dmg_name);
    let final_dmg_path = format!("{}/{}.dmg", output_dir, dmg_name);

    println!("════════════════════════════════════════════════════════");
    println!("Creating DMG for {} version {}", app_name, version);
    println!("════════════════════════════════════════════════════════");
    println!();

    // Check if app exists
    if !dry_run && !Path::new(app_path).exists() {
        return Err(anyhow::anyhow!("App not found at {}", app_path));
    }

    // Create output directory
    if !dry_run {
        std::fs::create_dir_all(output_dir)
            .context("Failed to create output directory")?;
    }

    // Step 1: Create temporary DMG
    println!("📦 Step 1: Creating temporary DMG...");
    let create_params = CommandParams::new("hdiutil", &[
        "create",
        "-size", "50m",
        "-fs", "HFS+",
        "-volname", &volume_name,
        &temp_dmg,
    ]);
    executor.execute(&create_params)?;

    // Step 2: Mount the temporary DMG
    println!("📂 Step 2: Mounting temporary DMG...");
    let mount_params = CommandParams::new("hdiutil", &[
        "attach",
        "-readwrite",
        "-noverify",
        "-noautoopen",
        &temp_dmg,
    ]);

    let mount_output = if !dry_run {
        executor.execute_with_output(&mount_params)?
    } else {
        "/Volumes/mock-mount".to_string()
    };

    // Extract mount point from hdiutil output
    // Format: /dev/disk4s2        Apple_HFS        /Volumes/Run IT 1.0
    let mount_dir = if !dry_run {
        mount_output
            .lines()
            .find(|line| line.contains("/Volumes/"))
            .and_then(|line| {
                // Find the index where "/Volumes/" starts and take everything from there
                line.find("/Volumes/").map(|idx| &line[idx..])
            })
            .ok_or_else(|| anyhow::anyhow!("Failed to parse mount point"))?
    } else {
        "/Volumes/mock-mount"
    };

    // Step 3: Copy the app bundle
    println!("📋 Step 3: Copying application...");
    let app_dest = format!("{}/", mount_dir);
    let cp_params = CommandParams::new("cp", &["-R", app_path, &app_dest]);
    executor.execute(&cp_params)?;

    // Step 4: Create Applications symlink
    println!("🔗 Step 4: Creating Applications symlink...");
    let symlink_params = CommandParams::new("ln", &[
        "-s",
        "/Applications",
        &format!("{}/Applications", mount_dir),
    ]);
    executor.execute(&symlink_params)?;

    // Step 5: Set DMG appearance with AppleScript
    println!("💅 Step 5: Setting DMG appearance...");
    let applescript = format!(r#"
        tell application "Finder"
            tell disk "{}"
                open
                set current view of container window to icon view
                set toolbar visible of container window to false
                set statusbar visible of container window to false
                set the bounds of container window to {{400, 100, 900, 440}}
                set viewOptions to the icon view options of container window
                set arrangement of viewOptions to not arranged
                set icon size of viewOptions to 72
                set position of item "{}.app" of container window to {{125, 150}}
                set position of item "Applications" of container window to {{375, 150}}
                update without registering applications
                delay 1
                close
            end tell
        end tell
    "#, volume_name, app_name);

    let osascript_params = CommandParams::new("osascript", &["-e", &applescript]);
    // Don't fail if AppleScript doesn't work (it's optional)
    if executor.execute(&osascript_params).is_err() {
        println!("⚠️  Could not set DMG appearance (this is optional)");
    }

    // Wait for Finder to finish
    println!("💤 Waiting for Finder to finish...");
    if !dry_run {
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    // Step 6: Unmount the temporary DMG
    println!("🔓 Step 6: Unmounting temporary DMG...");
    let detach_params = CommandParams::new("hdiutil", &["detach", mount_dir]);
    executor.execute(&detach_params)?;

    // Step 7: Convert to compressed DMG
    println!("🗜️  Step 7: Converting to compressed DMG...");
    let convert_params = CommandParams::new("hdiutil", &[
        "convert",
        &temp_dmg,
        "-format", "UDZO",
        "-o", &final_dmg_path,
    ]);
    executor.execute(&convert_params)?;

    // Step 8: Clean up temporary DMG
    println!("🧹 Step 8: Cleaning up...");
    if !dry_run {
        std::fs::remove_file(&temp_dmg)
            .context("Failed to remove temporary DMG")?;
    } else {
        println!("  Command: rm {}", temp_dmg);
    }

    println!();
    println!("✅ DMG created successfully!");
    println!("   Location: {}", final_dmg_path);
    println!();

    // Get file size
    if !dry_run {
        if let Ok(metadata) = std::fs::metadata(&final_dmg_path) {
            let file_size = metadata.len();
            println!("📊 File size: {} bytes", file_size);
            println!();
        }
    }

    Ok(final_dmg_path)
}
