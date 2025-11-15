use anyhow::Result;
use crate::commands::config::ReleaseConfig;
use std::collections::HashMap;
use std::fs;

pub fn parse_deployment_file(path: &str, dry_run: bool) -> Result<ReleaseConfig> {
    let deployment_content = fs::read_to_string(path)?;
    parse_deployment_config(&deployment_content, dry_run)
}

fn parse_deployment_config(content: &str, dry_run: bool) -> Result<ReleaseConfig> {
    let mut config_map = HashMap::new();

    // First pass: collect all variables as raw values
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim();
            let value = line[pos + 1..].trim();
            config_map.insert(key.to_string(), value.to_string());
        }
    }

    // Second pass: perform variable substitution
    let mut resolved_config = HashMap::new();
    for (key, value) in config_map.iter() {
        let resolved_value = perform_variable_substitution(value, &config_map);
        resolved_config.insert(key.clone(), resolved_value);
    }

    // Extract variables from the config
    let config = ReleaseConfig {
        app_name: resolved_config.get("APP_NAME").unwrap_or(&"".to_string()).clone(),
        version: resolved_config.get("VERSION").cloned(),
        archive_folder: resolved_config.get("ARCHIVE_FOLDER").unwrap_or(&"./archives".to_string()).clone(),
        scheme: resolved_config.get("SCHEME").unwrap_or(&"".to_string()).clone(),
        release_folder: resolved_config.get("RELEASE_FOLDER").unwrap_or(&"./releases".to_string()).clone(),
        apple_id: resolved_config.get("APPLE_ID").unwrap_or(&"".to_string()).clone(),
        team_id: resolved_config.get("TEAM_ID").unwrap_or(&"".to_string()).clone(),
        account_token: resolved_config.get("ACCOUNT_TOKEN").unwrap_or(&"".to_string()).clone(),
        export_method: resolved_config.get("EXPORT_METHOD").unwrap_or(&"app-store".to_string()).clone(),
        zip_name: resolved_config.get("ZIP_NAME").cloned(),
        export_options_plist: "ExportOptions.plist".to_string(),
        sparkle_host: resolved_config.get("SPARKLE_HOST").cloned(),
        dry_run,
    };

    Ok(config)
}

fn perform_variable_substitution(value: &str, variables: &HashMap<String, String>) -> String {
    let mut result = value.to_string();

    // Find and replace all ${VAR_NAME} patterns
    let mut start = 0;
    while let Some(pos) = result[start..].find("${") {
        let pos = start + pos;
        if let Some(end_pos) = result[pos..].find('}') {
            let full_end_pos = pos + end_pos;
            let variable_pattern = &result[pos..=full_end_pos];

            // Extract the variable name
            let var_name = &variable_pattern[2..variable_pattern.len()-1]; // Remove ${ and }

            // Look up the variable value
            if let Some(replacement) = variables.get(var_name) {
                result = format!("{}{}{}",
                    &result[..pos],
                    replacement,
                    &result[full_end_pos+1..]);

                // Update start to continue after the replacement
                start = pos + replacement.len();
            } else {
                // Variable not found, skip to after the pattern
                start = full_end_pos + 1;
            }
        } else {
            // No closing } found, exit the loop
            break;
        }
    }

    result
}

pub fn validate_config(config: &ReleaseConfig) -> Result<()> {
    let mut errors = Vec::new();

    if config.app_name.is_empty() {
        errors.push("APP_NAME is required in .deployment file");
    }

    if config.scheme.is_empty() {
        errors.push("SCHEME is required in .deployment file");
    }

    if config.apple_id.is_empty() {
        errors.push("APPLE_ID is required in .deployment file");
    }

    if config.team_id.is_empty() {
        errors.push("TEAM_ID is required in .deployment file");
    }

    if config.account_token.is_empty() {
        errors.push("ACCOUNT_TOKEN is required in .deployment file");
    }

    if config.export_method.is_empty() {
        errors.push("EXPORT_METHOD is required in .deployment file");
    }

    if !errors.is_empty() {
        eprintln!("Configuration errors found:");
        for error in errors {
            eprintln!("  - {}", error);
        }
        eprintln!();
        eprintln!("You can create a .deployment file based on the template:");
        eprintln!();
        print_deployment_template();
        std::process::exit(1);
    }

    Ok(())
}

fn print_deployment_template() {
    println!("{}", include_str!("../../template.deployment"));
}