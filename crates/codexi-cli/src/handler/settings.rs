use anyhow::Result;

use codexi::{core::DataPaths, dto::CodexiSettingsItem, file_management::FileManagement};

use crate::{command::SettingsCommand, msg_info, msg_warn, ui::view_codexi_settings};

pub fn handle_settings_command(command: SettingsCommand, paths: &DataPaths) -> Result<()> {
    let mut codexi = FileManagement::load_current_state(paths)?;
    match command {
        SettingsCommand::View => {
            let settings = CodexiSettingsItem::from(&codexi);
            view_codexi_settings(&settings);
        }
        SettingsCommand::Set { language, currency } => {
            if language.is_none() && currency.is_none() {
                msg_warn!("Codexi setting not set, no arguments provided");
            } else {
                let settings = codexi.set_settings(language.as_deref(), currency.as_deref())?;
                settings.save_default_path()?;
                FileManagement::save_current_state(&codexi, paths)?;
                msg_info!("Codexi setting set");
            }
        }
    }
    Ok(())
}
