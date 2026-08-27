# Changelog

All notable changes to this project will be documented in this file.

---
## [Unreleased] — 

### Added
- **Input normalization** – to sanitize CLI input from external sources (Unicode whitespace, control chars).
- **CLI command `--tui`** – a simple TUI for quick access of overviezw, view, report, account, counterparty, category, backup.
- **Codexi::Rebuild** – reconstruct the exchange rate on tranfer operation.
- **Cli Ui** – add a note in the report of the counterparty related to the tranfer operation`s.  
- **Cli Ui** – add counterparty in `operation view`.
- **Cli Ui** – add data direcroy in `admin infos`.

### Changed
- **Audit** – moved to codexi-level, refactored.
- **Storage** – only Ciborium format is now supported.
- **`view_account_context`, `view_category`** – improved views, minor fixes.
- ⚠️ **Search balance** – introduced filtered running balance (does not reflect global account/operation balance).
- ⚠️ **Backup** – replace ZIP archives with tar + gz compression
- **Cli Ui** – extract exchange rate display into dedicated 'rate' module ui/rate.rs"
- **Dependencies** – Moved `rust_decimal_macros` from `dependencies` to `dev-dependencies` to reduce production binary size. All `dec!` macro calls replaced with direct `Decimal` constructors (e.g., `dec!(500)` → `Decimal::new(500, 0)`, `dec!(0.001)` → `Decimal::new(1, 3)`).
- **CLI help messages** — improved clarity, consistency, and corrected wording across commands and arguments.
- **Backup system** – added manifest with version and application metadata, now backs up both data and config directories separately.
- **Path handling** – replaced directories with dirs crate, split data and config directories for better cross-platform compatibility.


### Removed
- ⚠️ **Support** for migration from v1 and v2 (legacy).
- ⚠️ **Support** of reading ledger format file (legacy) by `bincode`and `serde_cbor` crates.
- **CLI command `admin migrate`**
- **`bincode`, `serde_cbor`** crates.
- **`log`, `env_logger`** crates.

### Fixed
- Transfer link initialization by assigning `transfer_id` after destination operation creation.
- Unintended overwrite of custom exchange rates in OperationBuilder by only defaulting to `1` when the value is unset (`0`).

---
## [0.5.0] — 2026-04-13

### Added
- **CLI command `report rate`** — new exchange rate report with `--from` and `--to`
  date filters. Lists all operations with a meaningful exchange rate (`rate ≠ 1`),
  including transfers. Displays amount in account currency, effective rate, and cost
  in the target currency (`amount × rate`). Summary shows avg, best and worst rate
  over the period. Cost currency resolved automatically when exactly one other
  currency exists in the ledger.
- **CLI command `operation update --rate / -r`** — new flag accepting two values
  `<from> <to>` to set the exchange rate on an existing `Transaction` operation.
  Rate stored as `to / from` (units of target currency per unit of source currency).
  Rejected on non-Transaction kinds. Guards: `from` and `to` must both be strictly
  positive before division to avoid panic on zero.
- **CLI command `report tree`** — new tree view `Counterparty → Category → Operations`
  with `--from` and `--to` date filters. Displays hierarchical grouping of all active
  operations, including transfers and uncategorized entries under `[No Counterparty]`
  and `[No Category]`.
- **CLI command `operation update`** — `--description / -d`, `--counterparty / -c`
  and `--category / -g` flags to update an existing operation in place.
- **CLI command `search`** — `-c / --counterparty` and `-g / --category` flags wired
  through to the search layer. Passing `None` returns operations with no counterparty
  or no category respectively.
- **CLI command `report counterparty`** — support `--from` and `--to`.
- **CLI command `report category`** — support `--from` and `--to`.
- **CLI command `report monthly`** — support `--from` and `--to`.
- **CLI command `report dashboard`** — support `--from` and `--to`.
- **CLI command `use`** — alias for `account use`.
- **`AccountType::Cash`** — new account type for physical cash wallets and pass-through
  accounts. Compliance rules identical to `Current` (overdraft allowed by convention).
  `savings_rate` is not applicable and displays `N/A` in financial reports.
- **Logic `group_by_counterparty_category`** — new method on `SearchOperationList`
  producing a two-level `Counterparty → Category` grouping with nested operations.
  Transfers fall naturally under `[No Counterparty]`. Void and voided operations
  excluded from totals and grouping via `active_items`.
- **DTOs `CounterpartyTreeCollection` / `CounterpartyNode` / `CategoryNode` /
  `OperationLeaf`** — tree DTO layer for `report tree`. Follows the same
  `build(groups)` pattern as existing stats collections.
- **DTOs `CategoryItem::build`** — resolves parent data (name, terminated state).
- **DTOs `MonthlyReport::build`** — used by `report monthly`.
- **DTOs `CounterpartyStatsCollection::build`** — used by `report counterparty`.
- **DTOs `CategoryStatsCollection::build`** — used by `report category`.
- **DTOs `DashboardCollection::build`** — used by `report dashboard`.
- **UI `view_tree`** — tree view for `report tree`. Uses `├─` / `└─` indentation
  with debit/credit totals at counterparty and category level.
- **UI `view_monthly_report`** — view for `report monthly`.
- **UI `view_counterparty_stats`** — stats view for `report counterparty`.
- **UI `view_category_stats`** — stats view for `report category`.
- **UI `view_dashboard`** — dashboard view for `report dashboard`.

### Changed
- **`ExchangeRateCollection::build`** — cost computed as `amount × rate` (standard
  direct rate convention: `rate = to / from`, `cost = amount × rate = amount_target`).
  No flow-based branching needed — formula is correct for both debit and credit
  regardless of account currency direction.
- **CLI command `report stats`** — renamed to `report financial` to clarify purpose.
- **`active_items()`** — now excludes void and voided operations in addition to
  init and checkpoint. All grouping functions (`group_by_counterparty`,
  `group_by_category`, `group_by_counterparty_category`) benefit automatically —
  eliminates phantom rows with zero totals caused by void operations.
- **`group_by_counterparty`** — Transfer operations are no longer skipped via an
  explicit guard. They fall naturally under `[No Counterparty]` when no counterparty
  is assigned, consistent with the tree view behavior.
- **`StatsCollection`** — reworked financial analytics around two explicit perimeters:
  - *Perimeter 1 (account reality)* — all flows excluding init, checkpoint, void,
    voided. Used for `total_credit`, `total_debit`, `balance`, `operation_count`.
  - *Perimeter 2 (real economic flows)* — excludes Transfer DR (loans, internal
    moves). Transfer CR kept as available cash. Used for `savings_rate`,
    `daily_average`, `average_operation`, `top_expenses`, `max_single_debit`.
  - `savings_rate` is `Option<Decimal>` — `None` for `Cash` accounts.
  - `real_total_credit`, `real_total_debit`, `real_operation_count` added.
  - Transfers removed from `top 5 expenses` and `daily burning rate`.
- **`search/operation.rs`** — refactored grouping helpers:
  - Private helpers `resolve_counterparty_name_kind`, `resolve_category_name`,
    `accumulate_flow` extracted to eliminate duplication across the three
    `group_by_*` functions.
  - Constants `NO_COUNTERPARTY` and `NO_CATEGORY` replace scattered string literals.
  - Type aliases `CategoryInnerMap` and `CounterpartyOuterMap` added to satisfy
    `clippy::type_complexity` on the nested `HashMap` in
    `group_by_counterparty_category`.
- **UI `view_financial`** — updated to reflect two-perimeter model:
  - `ops count (all)` and `ops count (real)` shown separately.
  - `savings rate` hidden and replaced by `N/A` for `Cash` accounts.
  - Notes section displayed only when relevant (savings rate present or ignored
    operations exist).
  - Labels updated: `excl. transfer DR, adjust, void, voided` for behavioral
    insights and top expenses.
- **UI `view_dashboard`** — savings rate block handles `Option<Decimal>`:
  `Some(v)` renders value + bar over two lines, `None` renders
  `N/A (cash account)` + empty line to preserve column alignment.
- **UI `view_monthly_report`** — savings rate column handles `Option<Decimal>`:
  `None` renders `N/A` with `(not supported by account type)` annotation.
- **UI** — unified UI style constants and helpers introduced.
- **UI** — account switch messaging updated to include name and id
  (`Switched to account: <name> (<id>)`).
- **UI `view_financial` top expenses** — tie-breaker on `op_date` added for
  deterministic ordering of equal-amount expenses.
- **UI `view_category_list`** — enhanced to surface parent info and termination
  state.
- **DTOs** — `parent_name` and `parent_terminated` added to `CategoryItem`.
  `CategoryCollection::build` updated accordingly.
- **`ExchangeCategoryList.list`** — serde rename: serializes as JSON key
  `"categories"`.
- **`ExchangeCounterpartyList.list`** — serde rename: serializes as JSON key
  `"counterparties"`.
- **Import** — refactored import functionality out of `Codexi` model into
  `Codexi import` for better separation of concerns.
- **UI `view_codexi_infos`** — operation count now includes archived files.
- **UI `overview_account`** — indicator added for accounts expected to carry
  zero balance.

### Fixed
- **`ExchangeOperation`** — `account_id` field now exports `account.id` instead
  of `operation.id`.

## [0.4.0] — 2026-04-05
> ⚠️ Breaking: DTO modules have been moved from `logic/*` to `dto/*`

> ⚠️ Breaking: `Operation` structure, additional field `account_id`, performed the command `admin audit --rebuild`

### Added

- **CLI command `loan policy show`** — display current persisted loan policy.
- **CLI command `loan policy set`** — update one or more policy fields without
  overwriting unspecified fields. Flags: `--type`, `--rate`, `--free-days`,
  `--max-cap`, `--max-days`, `--min-capital`, `--max-penalty`.
- **CLI command `loan policy reset`** — reset policy to default values and persist.
- **CLI command `loan simulate`** — compute amount due for a given loan.
  Required: `--capital`, `--start`, `--refund`.
  Optional overrides (without modifying policy): `--type`, `--rate`, `--free-days`.
  Displays: amount due, total interest, first interest date, per-day interest breakdown.
- **CLI Command** — `overview` to display the main informations of the accounts, including the balance, account type, bank, and currency. 
- **CLI commands** — `data export category` and their `import` counterparts. Format argument: `json or toml`. CSV reserved, returns `UnsupportedFormat`.
- **CLI commands** — `data export couterparty` and their `import` counterparts. Format argument: `json or toml`. CSV reserved, returns `UnsupportedFormat`.
- **Account::set_context** as a wrapper around `context.update_context`.
- **termination guards** in `set_context`, `set_bank` and `set_currency` to prevent mutating terminated accounts.
- **`ignored`** operations list to stats output to surface operations excluded from calculations (e.g. unmatched void/voided pairs)
- **ExchangeCategoryList** — dedicated exchange DTO for the category list. Export produces `categories.json/toml`. Import merges categoris — new categories created, existing updated by id. Duplicate code against existing list rejected as error.
- **Import validation — categories** — `validate_import_categorie` covers: version, id format if present, name.
- **Seed** — default seed for counterparty are available at the creation of a new ledger.
- **Counterparty** — model, DTO, exchange import/export and validation wired end-to-end.
  Fields `id`, `name`, `kind` (`Person` / `Organisation`), `note`, `terminated`.
  Includes default seed at ledger creation and CLI subcommand `counterparty` (list, add, terminate).
- **`-c / --counterparty`** flag added to CLI commands `debit`, `credit`, `interest`.
  Accepts full id, short id, or name (exact / prefix / contains). Not available on `transfer` (inter-account operation).
- **`-g / --category`** flag added to CLI commands `debit`, `credit`, `interest`, `transfer`.
  Same resolution strategy as counterparty.
- **`counterparty_id` and `category_id`** added to `OperationContext` — carried through
  `register_transaction`, `transfer`, merge, audit and rebuild.
- **`ExchangeCounterparty`** — exchange DTO for counterparty import/export. Validation covers
  duplicate ids, unknown kind. Import creates or updates counterparties by id.
- **`logic/loan` module** — standalone calculation module for microloan tracking.
  No ledger integration — pure computation aid. Two interest models supported:
  - `LinearInterest` — simple daily interest: `capital × rate × late_days`
  - `CompoundInterest` — compound daily interest: `capital × ((1 + rate)^days - 1)`
  Both models share a common `LoanBase` with `LoanPolicy` (cap, penalty, min capital,
  max duration, free period). Cap applies to interest only; penalty is a fixed percentage
  of capital added after cap. `LoanSummary` returns `final_due`, `total_interest`,
  per-day interest vector with dates, and `first_interest_date`.
- **`LoanKind`** — enum (`Linear` / `Compound`) shared between domain model and settings.
  Replaces the former `LoanInterestType` in settings layer.
- **`LoanPolicySettings`** — persisted loan policy stored in `tmp/loan_policy.json`.
  Loaded via `load_or_create`, saved via `save`, reset to defaults via `reset`.
  Converted to domain `LoanPolicy` via `to_loan_policy()`.

### Changed
- **CLI handlers** to use new DTO construction patterns
- **CLI command** `search`, alias `view` now support the flag `--open` to open the result in the default browser.
- **Refactored DTO layer**:
  - Moved DTOs from `logic/*` into dedicated `dto/*` modules
  - Simplified DTOs to only handle basic type conversions (e.g. `NaiveDate`, `Nulid`, `Path` → `String`)
- **Moved formatting logic** (e.g. booleans, display helpers) from DTOs to CLI UI layer
- **Stats calculation** — Reworked logic to ensure consistency with account statements.
  - Stats now exclude init and checkpoint operations while including all other valid operations,
  - Introduced robust handling of void and voided operations: Operations are only included if both sides of the pair exist within the selected period,
  - Unmatched operations are excluded from financial calculations,
  - Improved accuracy of financial indicators (balance, totals, savings rate, averages, daily burn rate),
  - Refactored stats computation pipeline for clarity, consistency, and performance (single-pass processing).
- **`account_id`** added to `Operation` structure — existing data requires `admin audit --rebuild`.
- **`Account::set/merge/audit`** updated to handle legacy operations missing `account_id`
  and to populate it during audit/rebuild.

### Removed
- **`AccountBalance`** and **`CodexiBalance`** from Balance. 
- **`signed_amount`** function in `logic/operation/model.rs`.
- **CLI Command** - `report balance-all`. Replace by the command `òverview`.
- **Formatting responsibilities** from DTOs (now handled exclusively in UI)
- Replaced legacy `entry.rs` mapping functions with DTO-based conversions
- **CLI command** `report stats`, the flag `--net` is not more available 


## [0.3.0] — 2026-03-28

### Added
- **ExchangeAccountHeader** — dedicated exchange DTO for account metadata (name, context, bank, currency, dates, meta). Replaces the monolithic `ExchangeData`. Export produces `account-header.json/toml`, import creates or updates account metadata only — operations untouched.
- **ExchangeAccountOperations** — dedicated exchange DTO for operations tied to an account (`account_id` + `Vec<ExchangeOperation>`). Export produces `operations.json/toml`. Import merges operations into the target account identified by `account_id` — account must exist, terminated accounts rejected.
- **ExchangeCurrencyList** — dedicated exchange DTO for the currency list. Export produces `currencies.json/toml`. Import merges currencies — new currencies created, existing updated by id. Duplicate code against existing list rejected as error.
- **AccountOperations** — new domain struct in `logic/operation` grouping `account_id` and `Vec<Operation>`. Used as the unit of exchange for operation import/export. `Account::to_account_operations()` builds a snapshot for export.
- **Exchangeable** trait — single generic trait replacing the former `JsonExchange` + `TomlExchange` duplication. Each exchangeable type declares its `Exchange` DTO, `exchange_filename`, `to_exchange` and `from_exchange`. Implemented for `Account`, `CurrencyList`, `AccountOperations`. 
- **ExchangeSerdeFormat** — central dispatch for serde I/O. Single place in the codebase that knows about `serde_json` and `toml. Bridges `ExchangeError` (domain)` → `FileExchangeError` (I/O) via `From`.
- **ExchangeBase trait** — associates a `Warning` type to each exchangeable domain type. Currently `CoreWarning` for all implementations.
- **Import validation — operations** — `validate_import_operations` covers: version, account id format, duplicate operation ids, strictly positive amounts, transfer link consistency (both fields or neither), transfer kind coherence, void consistency (void must reference, no double-void, no void-of-void). Operations without id (new) only allowed without void/transfer links.
- **Import validation — currencies** — `validate_import_currency` covers: version, id format if present, non-empty code, ISO 4217 length (exactly 3 chars), duplicate codes within the file.
- **Import validation — account header** — `validate_import_account_header` covers: version, non-empty name, known `account_type`.
- **Identity resolution on currency import** — `CurrencyList::resolve_id` resolves incoming currency identity: known id → reuse, unknown id but matching code → reuse existing id, no match → generate fresh Nulid. Prevents duplicate currencies across repeated imports.
- **Merge** — **currencies** — `merge_from_import` updates `symbol` and `note` on existing currencies (by id). New currencies added via `add()`. Duplicate code against existing list returns `CurrencyError::DuplicateCode` — full import rejected.
- **Merge** — **account header** — `merge_account_header_from_import` updates `name`, `currency_id`, `bank_id`, `context`, `meta`. Terminated accounts rejected. `carry_forward_balance`, `open_date`, `anchors`, `checkpoints` never updated via import.
- **Merge** — **operations** — `merge_operation_from_import` updates `description`, `context` (category, payee, reconciled), `meta` on existing operations. New `Transaction` and `Init` operations added. Unsupported kinds (`Void`, `Transfer`, `Adjust`) skipped with `CoreWarning`. `refresh_anchors() always called after merge.
- **CLI commands** — `codexi-cli data export account-header`, `codexi-cli data export operation`, `codexi-cli data export currency` and their `import` counterparts. Format argument: `json or toml`. CSV reserved, returns `UnsupportedFormat`.
- **`FileExchangeError::UnsupportedFormat`** — returned when CSV format requested for import/export.
- **`resolve_or_generate_id`** helper in `core/parse.rs` — resolves `Option<&str>` to `Nulid`: parses if present (expects prior validation), generates fresh Nulid if absent.
- **`default_zero`** serde helper — used for optional decimal string fields (`carry_forward_balance`, `current_balance`, `overdraft_limit`, `min_balance`) defaulting to `"0"` when absent.
- **Accountype** — New account type for loan, Income context and policy updated accordingly
- **`HasName` trait** — optional trait in `logic/utils` decoupled from `HasNulid`. Implemented on types that carry a human-readable name field (currently `Account`). Types without a name (`Operation`, `Transfer`, …) are unaffected and continue to use `resolve_id`.
- **`resolve_by_id_or_name`** — new generic resolver in `logic/utils` combining ID-based and name-based lookup in a single call. Resolution priority: full Nulid (26 chars) → ID suffix → name exact → name prefix → name contains. Ambiguity is evaluated independently at each level — a unique exact match wins regardless of prefix/contains candidates. Available only for types implementing both `HasNulid` and `HasName`.
- **`resolve_by_name`** (internal) — three-tier name search (exact / prefix / contains, case-insensitive) extracted as a private helper. Keeps `resolve_by_id_or_name` readable and allows independent testing of the name tier.
- **`match_unique`** (internal) — shared finalizer converting a candidate `Vec<&T>` into `Ok(Nulid)`, `Err::not_found`, or `Err::ambiguous`. Eliminates duplicated match-count logic across resolution tiers.
- **CLI command `account use`** — now accepts account name (or prefix/substring) in addition to full or short ID. `account use perso`, `account use wall`, `account use alc` all resolve correctly. Ambiguous input (e.g. `al` matching multiple accounts) returns a clear error prompting for more characters.
- **CLI command `account set-bank`** — now accepts <bank_name> in addition to <bank_id>.
- **CLI command `account set-currency`** — now accepts <currency_code> in addition to <currency_id>.
- **CLI command `transfer`** — <account_id_to> argument now accepts an account name (or prefix/substring) in addition to full or short ID.
- **`CodexiBalanceEntry`** — in `logic/balance/model.rs` to compute balance, debit, credit of the accounts.
- **CLI command `report balance-all`** — display the balance, debit and credit per accounts from `CodexiBalanceEntry`.
- **`RegularKind::Interest`** — new operation kind for interest accrual and cash receipt. Semantically distinct from `Transaction` — enables filtering, reporting, and enforcement of `allows_interest` per account. Used with `Credit` flow to record interest due (option B: accrual as additional debt), voided via `SystemKind::Void` for interest forgiveness.
- **`ComplianceViolation::KindNotAllowed(AccountType)`** — new typed error replacing the former `NotAllowed { reason }` catch-all. Carries the account type for clear error messages.
- **`ComplianceViolation::InitNonZeroOnLoan`** — new typed error for `Init` operations with a non-zero amount on `Loan` accounts.
- **`AccountType::allows_interest()`** — new method returning `true` for `Saving`, `Deposit`, `Income` and `Loan`. Used as default initializer for `AccountContext::allows_interest` in `from_type()`.
- **`AccountType::allows_joint_signers()`** — new method returning `true` for `Joint` and `Business`. Used as default initializer for `AccountContext::allows_joint_signers` in `from_type()`.
- **`AccountContext::from_type()`** — now uses `allows_interest()` and `allows_joint_signers()` methods for initialization. `Loan` and `Income` defaults to `allows_interest: true`. No more hardcoded booleans.
- **`validate_full` signature** — now receives `_kind: &RegularKind` and `_flow: OperationFlow` to allow per-type kind/flow guards in overrides without duplicating the dispatch logic from `validate()`.
- **`LoanPolicy`** — full `validate()` override. Blocks `Init` with non-zero amount (`InitNonZeroOnLoan`). Restricts Regular ops to `Transfer` (both flows), `Interest Credit` (if `allows_interest`), and `Fee` (both flows). All others return `KindNotAllowed`.
- **`SavingPolicy`** — `validate_full` override blocks `Transfer Debit` explicitly (`KindNotAllowed`) regardless of balance. All other debits blocked by `validate_no_overdraft`.
- **`DepositPolicy`** — all debits blocked before `deposit_locked_until` via `NoWithdrawalAllowed`. Credits always allowed. No overdraft after maturity.
- **`validate_no_overdraft`** — shared private helper now used by `SavingPolicy`, `DepositPolicy`, `LoanPolicy` and `IncomePolicy`. Returns `NegativeBalanceNotAllowed(AccountType)`.
- **Compliance matrix** — full kind × flow × account-type decision table established as reference. Covers all `RegularKind` (Transaction, Transfer, Interest, Fee, Refund) and `SystemKind` (Init, Adjust, Checkpoint, Void) combinations across all eight account types.
- **`AccountType::Income`** — new account type for pure accumulation (interest income, revenue pools). Transfer only in both directions, no overdraft, no negative balance. Replaces `Deposit` as the recommended type for interest income accounts.
- **`IncomePolicy`** — compliance policy for `Income` accounts. Only `Transfer` (Credit/Debit) allowed. All other Regular kinds (`Transaction`, `Fee`, `Refund`, `Interest`) return `KindNotAllowed`.
- **`docs/loan.md`** — user guide for microloan management: account setup, operation cycle, interest accrual pattern, full worked example with balance states at each step.
- **`docs/compliance_matrix.md`** — reference table of all allowed/refused/conditional operations by kind, flow, and account type.
- **`docs/context_matrix.md`** — reference table of all configurable context fields by account type with default values.

### Changed
- **ExchangeData removed** — replaced by `ExchangeAccountHeader` + `ExchangeAccountOperations`. Split clarifies responsibilities and produces smaller, more readable export files.
- **ImportSummary.account_name** renamed to **name** — field is now used for any imported entity (account, currencies), not accounts only.
- **export_toml** / **export_json** / **import_toml** / **import_json** — now generic over `T: Exchangeable`. Format dispatch centralized in `ExchangeSerdeFormat`, `json.rs` and `toml.rs` are thin wrappers.
- **import_operations** in **Codexi** — returns `Result<(ImportSummary, Vec<CoreWarning>), CodexiError>` — merge warnings (skipped kinds) propagated to CLI.
- **Validation module** — `validation.rs` renamed to `validator/` with per-entity files (`account.rs, currency.rs, operation.rs)`. `validate_import` split into `validate_import_account_header`, `validate_import_currency`, `validate_import_operations`.
- **CLI commands** — `account list` and `account context` output view updated to be more friendly.
- **`ComplianceAction`** — `Create` variant now owns `OperationKind` and `OperationFlow` by value (was `&'a OperationKind`). Lifetime removed — `OperationKind` is `Copy`.
- **`ComplianceViolation::NotAllowed`** — removed. Replaced by typed variants `KindNotAllowed(AccountType)` and `InitNonZeroOnLoan`. Error messages are now generated by `thiserror` from the type, not from a `&'static str` field.
- **`AccountType::Display`** — padding (`{:<7}`) removed from `Display` implementation. Alignment is now the responsibility of the CLI display layer, not the domain type.
- **`validate_no_overdraft`** — extracted as a shared private function for all no-overdraft policies. Previously duplicated between `LoanPolicy` and `SavingPolicy`.
- **CLI commands** — `admin export-script` take into account the command `transfer` and `interest` in the script generation.

### Fixed
- **export_json** wrote **json.as_bytes()** — now writes `String` directly via `fs::write`, removing the unnecessary `.as_bytes()` conversion.
- **Account::new()** — `account.context` was not properly set as per the account type. It now calls `AccountContext::from_type(account_type)` so the context is initialized correctly for the given account type

---

## [0.2.0] — 2026-03-23
> ⚠️ No automatic migration — use `admin export-script` from previous release before update to this release to rebuild your data.

### Added
- **AccountContext** — configurable parameters per account instance (overdraft limit,
  minimum balance, monthly transaction quota, deposit lock date, interest, joint signers).
  Each account starts from type defaults and can be individually adjusted.
- **CompliancePolicy** — per-account-type business rules validation (overdraft, minimum
  balance, monthly quota, deposit maturity lock). Each account type has its own policy
  implementation (Current, Saving, Joint, Deposit, Business, Student). Values are read
  from `AccountContext` — never hardcoded.
- **LifecyclePolicy** — account lifecycle rules: account type is immutable once operations
  exist; close date validated against today, `open_date`, and last operation date.
- **TemporalPolicy** — renamed and clarified from the previous `FinancialPolicy`.
  Handles chronological and structural rules on operations (date ordering, period locking,
  init/void/checkpoint sequencing). Orthogonal to `CompliancePolicy`.
- **`LastAnchor` struct** — replaces bare `Option<NaiveDate>` in `AccountAnchors`.
  Each anchor now stores both `date` and `id` (Nulid), enabling precise same-day
  ordering for void and lock decisions. Applied uniformly to all anchor types
  (`last_init`, `last_adjust`, `last_checkpoint`, `last_void`, `last_regular`).
- **Generic ID resolution (`resolve_id`)** — reusable mechanism to resolve full or short
  IDs (Nulid) across entities (operations, accounts, etc.). Supports suffix-based lookup
  with configurable minimum length (`MIN_SHORT_LEN`).
- **Traits `HasNulid` and `ResolveError`** — enable generic ID resolution by decoupling
  entity identification and error handling from business logic. Allows reuse across
  multiple domains (Account, Bank, Currency, Category, ...).
- **Transfer between accounts** — creates two linked `Regular::Transfer` operations
  (Debit on source, Credit on destination). Always operates from the current account.
  Exchange rate is calculated automatically as `amount_to / amount_from` — net effective
  rate including all fees, no manual input required.
- **Cross-currency transfer** — supports transfers between accounts with different
  currencies (e.g. EUR → IDR). The net exchange rate reflects the real cost of the
  transfer, fees included.
- **Transfer void** — voiding a transfer operation automatically voids both linked
  operations atomically. Uses the existing `void` command transparently — no separate
  command needed. Void is rejected if the twin operation is archived.
- **Cross-linked transfer references** — each transfer operation carries `transfer_id`
  (twin operation) and `transfer_account_id` (twin account) in `OperationLinks`,
  following the same symmetric pattern as `void_of`/`void_by`.
- **Short ID support in CLI commands** — entities can now be referenced using the last
  characters of their ID instead of the full 26-character Nulid.
- **Import validation hardened** — `validate_import` now rejects negative or zero amounts
  (`InvalidAmount`) and broken transfer links (`BrokenTransferLink`). Cross-account
  transfer references produce a warning rather than an error, as the twin operation
  belongs to another account.
- **Anchors always recalculated on import** — imported `AccountAnchors` are discarded
  and rebuilt from scratch via `refresh_anchors()`. Prevents stale or corrupted anchor
  data from silently propagating.
- **`OperationFlow::apply`** — new helper method eliminating duplicated `match flow`
  patterns across `rebuild_balances_from`, `balance_at`, and related balance calculations.
- **`OperationDetailItem`** — new rich DTO for single operation detail view. All
  referenced fields resolved (currency name, category name, transfer links). Built by
  `Codexi::operation_detail()`.
- **DTO / Entry layer refactored** — clean separation across all domains:
  `dto.rs` holds structures and `From` conversions only; `entry.rs` holds construction
  logic on `impl Account` or `impl Codexi`. `StatementEntry` moved to `codexi/` layer
  as it requires bank and currency resolution from `Codexi`.
- **`AccountAnchorsItem` and `SummaryEntry`** moved to `account/dto.rs` — consistent
  with the DTO pattern, away from `reports.rs`.
- **`StatsEntry` analytics** moved to `impl Account::stats_entry()` in `reports.rs` —
  clearly separated as analytical computation, not a simple DTO conversion.
- Command `account set-context` to update the configurable parameters of the current account.
- Command `account context` to view the context of the current account.
- Command `account create` now accepts an optional account type argument
  (Current, Joint, Saving, Deposit, Business, Student). Defaults to Current.
- Command `transfer <DATE> <AMOUNT_FROM> <AMOUNT_TO> <ACCOUNT_ID_TO> [DESCRIPTION]`
- Command `void` now handles both regular operations and transfer operations transparently.
- Command `operation view <ID> [--raw]` — detailed single operation view with resolved
  fields, color-coded flow, `[can void]` indicator, and optional raw debug output.

### Changed
- Command `account list` now shows the account type.
- Command `account -set-currency [--update-operation]`, option --update-operation to update the existing operation to the account currency.
- Refactored ID resolution logic — moved from account-specific implementation to a
  generic utility (`resolve_id`), improving consistency and reducing duplication.
- Commands such as `account use` and `history void` now support short ID input with
  validation (minimum length and ambiguity detection).
- **`compliance_policy` now takes an explicit `date` parameter** — balance validation
  uses `balance_at(date)` instead of `current_balance`, ensuring operations inserted at
  a past date are validated against the correct historical balance. Also fixes
  `monthly_operation_count` to count operations at the correct month.
- **Adjust lock granularity** — void of an operation on the same day as an adjust is now
  resolved by Nulid ordering: operations inserted before the adjust are locked, operations
  inserted after are allowed. Previously all same-day operations were incorrectly allowed.
- **`merge_from_import` now uses `commit_operation` directly** — new operations from
  import are inserted without re-running temporal or compliance policy, as validation
  is handled upstream by `validate_import`. Prevents incorrect rejection of valid
  historical data during merge.
- **Storage format** — write now uses `Ciborium` (official `serde_cbor` successor,
  maintained by the serde team). Read supports both `Cbor` and `Ciborium` for backward
  compatibility.
- **`Account::summary_entry`** now returns `SummaryEntry` directly (not `Option`) —
  a summary is always meaningful even for an empty period, as anchors are account-level.

### Deprecated
- `serde_cbor` — will be removed in a future version once no legacy `Cbor` files remain.

### Removed
- `OperationEntry::new` and `StatementEntry::new` as standalone constructors — replaced
  by `Account::operation_entry()` and `Codexi::statement_entry()` respectively.

### Fixed
- **Compliance validation on past-dated operations** — `compliance_policy` previously
  used `current_balance` for all validations, allowing overdraft violations when
  operations were inserted at a past date. Now uses `balance_at(date)` for accurate
  historical balance at the operation date.
- **Same-day adjust lock bypass** — operations on the same day as an adjust could
  incorrectly bypass the void lock. Now resolved precisely using Nulid ordering.
- **Audit TEST 8** — transfer link consistency check added: operations with `transfer_id`
  must also carry `transfer_account_id`, and must use `Regular::Transfer` kind.
- Option `--open` for commands `report stats` and `report statement` that did not open
  the default browser in some cases.

---

## [0.1.0] — 2026-03-16

*First release of the redesigned workspace version.*

### Added
- **Multi-account support** — create, list, rename, close accounts; switch active account with `account use`
- **Balance integrity** — `op.balance` stored on each operation for auditability with `rebuild_balances_from` for correction
- **Audit command** — `admin audit [--rebuild]` with policy replay, balance cross-reference, void link checks
- **Structured reporting** — `statement`, `stats`, `summary` with HTML export
- **AccountAnchors** — consolidated last dates (init, checkpoint, adjust, void, regular) as a dedicated struct
- **Counts module** — `src/logic/counts` shared across views
- **Script export** — `admin export-script` generates a replayable shell script for each account

### Changed
- **Workspace structure** — project split into two crates: `codexi` (core lib) and `codexi-cli`
- **Operation sorting** — unified sort by `(date, nulid)` in `commit_operation`
- **Balance calculation** — removed incremental `calculate_current_balance`, replaced by `rebuild_balances_from`

### Removed
- Export/import CSV format *(to be reintroduced in a future version)*

---

## Previous versions (legacy mono-crate)
### [2.0.1] — 2026-02-09
- Mono-crate application (`codexi`)

---
