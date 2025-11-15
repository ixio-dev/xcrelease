#[derive(Debug)]
pub struct ReleaseConfig {
    pub app_name: String,
    pub version: Option<String>,
    pub archive_folder: String,
    pub scheme: String,
    pub release_folder: String,
    pub apple_id: String,
    pub team_id: String,
    pub account_token: String,
    pub export_method: String,
    pub zip_name: Option<String>,
    pub export_options_plist: String,
    pub sparkle_host: Option<String>,
    pub dry_run: bool,
}