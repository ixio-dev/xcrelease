mod config;
mod settings_parser;
mod release;
mod template;
mod completion;
mod command_execution;
mod dmg;
mod sync;
mod version;

pub use release::handle_release;
pub use template::handle_template;
pub use completion::handle_completion;