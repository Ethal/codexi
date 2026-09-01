use crate::logic::codexi::Codexi;

#[derive(Debug)]
pub struct CodexiSettingsItem {
    pub language: String,
    pub default_currency: String,
}

impl From<&Codexi> for CodexiSettingsItem {
    fn from(codexi: &Codexi) -> Self {
        Self {
            language: codexi.settings.language.clone(),
            default_currency: codexi.settings.default_currency.clone(),
        }
    }
}
