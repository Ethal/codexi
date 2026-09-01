// src/logic/codexi/model.rs

use chrono::NaiveDate;
use nulid::Nulid;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    core::format_id,
    logic::{
        account::{Account, AccountError},
        bank::BankList,
        category::CategoryList,
        codexi::{
            CodexiError, CodexiSettings, default_banks, default_categories, default_counterparties, default_currencies,
        },
        counterparty::CounterpartyList,
        currency::CurrencyList,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Codexi {
    pub id: Nulid,
    pub name: String,
    pub settings: CodexiSettings, // Preferences (language, theme...)
    pub current_account: Nulid,

    // Dictionnary
    pub categories: CategoryList,
    pub banks: BankList,
    pub currencies: CurrencyList,
    #[serde(default)]
    pub counterparties: CounterpartyList,

    pub note: Option<String>,

    // The accounts
    pub accounts: Vec<Account>,
}

impl Codexi {
    pub fn new(settings: CodexiSettings) -> Result<Self, CodexiError> {
        let id = Nulid::new()?;

        // Seed data as per language settings
        let categories = CategoryList::from(default_categories(&settings.language));
        let counterparties = CounterpartyList::from(default_counterparties(&settings.language));
        let currencies = CurrencyList::from(default_currencies());
        let banks = BankList::from(default_banks());

        let name = "Codexi".into();
        let accounts = Vec::new();

        Ok(Self {
            id,
            name,
            settings,
            categories,
            banks,
            currencies,
            counterparties,
            accounts,
            current_account: Nulid::nil(),
            note: None,
        })
    }
    pub fn set_settings(
        &mut self,
        language: Option<&str>,
        currency: Option<&str>,
    ) -> Result<CodexiSettings, CodexiError> {
        self.settings.default_currency = currency.unwrap_or(&self.settings.default_currency).to_string();
        self.settings.language = language.unwrap_or(&self.settings.language).to_string();
        let settings = CodexiSettings {
            language: self.settings.language.clone(),
            default_currency: self.settings.default_currency.clone(),
        };
        Ok(settings)
    }
    pub fn get_account_by_id(&self, id: &Nulid) -> Result<&Account, CodexiError> {
        self.accounts
            .iter()
            .find(|c| &c.id == id)
            .ok_or_else(|| CodexiError::AccountNotFound(format_id(*id)))
    }
    pub fn get_account_by_id_mut(&mut self, id: &Nulid) -> Result<&mut Account, CodexiError> {
        self.accounts
            .iter_mut()
            .find(|c| &c.id == id)
            .ok_or_else(|| CodexiError::AccountNotFound(format_id(*id)))
    }
    pub fn set_current_account(&mut self, id: &Nulid) -> Result<(), CodexiError> {
        if self.accounts.iter().any(|a| &a.id == id) {
            self.current_account = *id;
            Ok(())
        } else {
            Err(CodexiError::AccountNotFound(format_id(*id)))
        }
    }
    pub fn get_current_account(&self) -> Result<&Account, CodexiError> {
        if self.current_account == Nulid::nil() {
            return Err(CodexiError::NoCurrentAccount);
        }
        self.accounts
            .iter()
            .find(|c| c.id == self.current_account)
            .ok_or_else(|| CodexiError::AccountNotFound(format_id(self.current_account)))
    }
    pub fn get_current_account_mut(&mut self) -> Result<&mut Account, CodexiError> {
        if self.current_account == Nulid::nil() {
            return Err(CodexiError::NoCurrentAccount);
        }
        self.accounts
            .iter_mut()
            .find(|c| c.id == self.current_account)
            .ok_or_else(|| CodexiError::AccountNotFound(format_id(self.current_account)))
    }

    /// Set the bank id to the current account
    pub fn set_account_bank(&mut self, id: &Nulid) -> Result<(), CodexiError> {
        self.banks.get_by_id(id)?;
        let acc = self.get_current_account_mut()?;
        acc.is_terminated().map_err(AccountError::TemporalViolation)?;
        acc.bank_id = Some(*id);
        Ok(())
    }
    /// Set a currency id to the current account
    pub fn set_account_currency(&mut self, id: &Nulid, update_operation: bool) -> Result<(), CodexiError> {
        self.currencies.get_by_id(id)?;
        let acc = self.get_current_account_mut()?;
        acc.is_terminated().map_err(AccountError::TemporalViolation)?;
        acc.currency_id = Some(*id);
        if update_operation {
            for op in acc.operations.iter_mut() {
                op.context.currency_id = Some(*id);
            }
        }
        Ok(())
    }
    /// Close an account
    /// Allowed if not the latest open account, current account, close date below the open_date
    pub fn close_account(&mut self, id: Nulid, date: NaiveDate) -> Result<(), CodexiError> {
        // Life cycle rules
        let account = self.get_account_by_id_mut(&id)?;
        account
            .validate_close_date(date)
            .map_err(AccountError::LifecycleViolation)?;

        // Apply the terminated date
        account.terminated_date = Some(date);

        // codexi rules
        // Switch current_account to nil if it was the current account
        if id == self.current_account {
            self.current_account = Nulid::nil();
        }

        Ok(())
    }

    /// Add an account
    /// Return the id of the new account
    pub fn add_account(&mut self, account: Account) -> Nulid {
        let id = account.id;
        self.current_account = id;
        self.accounts.push(account);
        id
    }

    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    pub fn update_operation(
        &mut self,
        op_id: Nulid,
        desc: Option<&str>,
        counterparty: Option<Nulid>,
        category: Option<Nulid>,
        from: Option<Decimal>,
        to: Option<Decimal>,
    ) -> Result<(), CodexiError> {
        let acc = self.get_current_account_mut()?;
        let op = acc
            .get_operation_by_id_mut(op_id)
            .ok_or(CodexiError::NoOperation(format_id(op_id)))?;
        if let Some(d) = desc {
            op.update_description(d);
        }
        if let Some(c) = counterparty {
            op.update_counterparty(c);
        }
        if let Some(g) = category {
            op.update_category(g);
        }
        if let Some(f) = from
            && let Some(t) = to
        {
            op.update_exchange_rate(f, t).map_err(AccountError::Operation)?;
        }
        Ok(())
    }
}
