// src/logic/search/operation.rs

use chrono::NaiveDate;
use derive_builder::Builder;
use nulid::Nulid;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::logic::{
    account::OperationContainer,
    category::CategoryList,
    counterparty::CounterpartyList,
    operation::{Operation, OperationFlow, OperationKind},
    search::SearchError,
    utils::HasNulid,
};

/*==============================================================================*/
/*                              CONSTANTS                                        */
/*==============================================================================*/

const NO_COUNTERPARTY: &str = "[No Counterparty]";
const NO_CATEGORY: &str = "[No Category]";

/*==============================================================================*/
/*                              PRIVATE HELPERS                                  */
/*==============================================================================*/

fn resolve_counterparty_name_kind(cp_id: Option<Nulid>, counterparties: &CounterpartyList) -> (String, String) {
    match cp_id {
        Some(id) => {
            let cp = counterparties.get_by_id(&id).ok();
            (
                cp.map(|c| c.name.clone()).unwrap_or_default(),
                cp.map(|c| c.kind.as_str().to_string()).unwrap_or_default(),
            )
        }
        None => (NO_COUNTERPARTY.to_string(), String::new()),
    }
}

fn resolve_category_name(cg_id: Option<Nulid>, categories: &CategoryList) -> String {
    match cg_id {
        Some(id) => {
            let cg = categories.get_by_id(&id).ok();
            cg.map(|c| c.name.clone()).unwrap_or_default()
        }
        None => NO_CATEGORY.to_string(),
    }
}

fn accumulate_flow(
    op: &Operation,
    total_debit: &mut Decimal,
    total_credit: &mut Decimal,
    last_date: &mut Option<NaiveDate>,
) {
    if op.flow.is_debit() {
        *total_debit += op.amount;
    } else {
        *total_credit += op.amount;
    }
    let d = op.date;
    *last_date = Some(match last_date {
        Some(existing) if *existing >= d => *existing,
        _ => d,
    });
}

/*==============================================================================*/
/*                              SEARCH OPERATION                                 */
/*==============================================================================*/

/// Struct for search operation
#[derive(Debug, Clone)]
pub struct SearchOperation {
    pub operation: Operation,
    pub balance: Decimal,
}

/// Struct for search operation list
#[derive(Debug, Clone)]
pub struct SearchOperationList {
    pub items: Vec<SearchOperation>,
    pub params: SearchParams,
}

impl SearchOperationList {
    pub fn new(params: &SearchParams) -> Self {
        Self {
            items: Vec::new(),
            params: params.clone(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &SearchOperation> {
        self.items.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Return a SearchOperation or None
    pub fn get_search_operation_by_id(&self, id: Nulid) -> Option<&SearchOperation> {
        self.items.iter().find(|so| so.operation.id == id)
    }
    /// Return operation list without the Init and the Checkpoint
    pub fn no_structural_items(&self) -> impl Iterator<Item = &SearchOperation> {
        self.items.iter().filter(|i| !i.operation.kind.is_structural())
    }

    /// Return operation list without the Init, Checkpoint, Void and Voided
    pub fn active_items(&self) -> impl Iterator<Item = &SearchOperation> {
        self.items
            .iter()
            .filter(|i| !i.operation.kind.is_structural() && !i.operation.is_void() && !i.operation.is_voided())
    }

    pub fn is_empty_active(&self) -> bool {
        self.active_items().next().is_none()
    }

    pub fn last_n(&self, n: usize) -> Vec<SearchOperation> {
        let len = self.items.len();
        let start = len.saturating_sub(n);
        self.items[start..].to_vec()
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }
}

impl Default for SearchOperationList {
    fn default() -> Self {
        Self::new(&SearchParams::default())
    }
}

impl HasNulid for SearchOperation {
    fn id(&self) -> Nulid {
        self.operation.id
    }
}

/*==============================================================================*/
/*                              CATEGORY                                        */
/*==============================================================================*/

#[derive(Debug, Clone)]
pub struct CategoryGroup {
    pub id: Option<Nulid>,
    pub name: String,
    pub op_count: usize,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub last_date: Option<NaiveDate>,
}

impl SearchOperationList {
    pub fn group_by_category(&self, categories: &CategoryList) -> Vec<CategoryGroup> {
        let mut map: HashMap<Option<Nulid>, CategoryGroup> = HashMap::new();

        for item in self.active_items() {
            let op = &item.operation;
            let cg_id = op.context.category_id;
            let entry = map.entry(cg_id).or_insert_with(|| CategoryGroup {
                id: cg_id,
                name: resolve_category_name(cg_id, categories),
                op_count: 0,
                total_debit: Decimal::ZERO,
                total_credit: Decimal::ZERO,
                last_date: None,
            });

            entry.op_count += 1;
            accumulate_flow(
                op,
                &mut entry.total_debit,
                &mut entry.total_credit,
                &mut entry.last_date,
            );
        }

        let mut groups: Vec<CategoryGroup> = map.into_values().collect();
        groups.sort_by(|a, b| {
            b.total_debit
                .partial_cmp(&a.total_debit)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        groups
    }
}

/*==============================================================================*/
/*                              COUNTERPARTY                                     */
/*==============================================================================*/

#[derive(Debug, Clone)]
pub struct CounterpartyGroup {
    pub id: Option<Nulid>,
    pub name: String,
    pub kind: String,
    pub op_count: usize,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub last_date: Option<NaiveDate>,
}

impl SearchOperationList {
    pub fn group_by_counterparty(&self, counterparties: &CounterpartyList) -> Vec<CounterpartyGroup> {
        let mut map: HashMap<Option<Nulid>, CounterpartyGroup> = HashMap::new();

        for item in self.active_items() {
            let op = &item.operation;
            // As transfer have no counterparty, transfer operation not part of the group
            if op.is_transfer() {
                continue;
            }
            let cp_id = op.context.counterparty_id;
            let entry = map.entry(cp_id).or_insert_with(|| {
                let (name, kind) = resolve_counterparty_name_kind(cp_id, counterparties);
                CounterpartyGroup {
                    id: cp_id,
                    name,
                    kind,
                    op_count: 0,
                    total_debit: Decimal::ZERO,
                    total_credit: Decimal::ZERO,
                    last_date: None,
                }
            });

            entry.op_count += 1;
            accumulate_flow(
                op,
                &mut entry.total_debit,
                &mut entry.total_credit,
                &mut entry.last_date,
            );
        }

        let mut groups: Vec<CounterpartyGroup> = map.into_values().collect();
        groups.sort_by(|a, b| {
            b.total_debit
                .partial_cmp(&a.total_debit)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        groups
    }
}

/*==============================================================================*/
/*                  TREE COUNTERPARTY / CATEGORY / OPERATION                    */
/*==============================================================================*/

type CategoryInnerMap = HashMap<Option<Nulid>, CategorySubGroup>;
type CounterpartyOuterMap = HashMap<Option<Nulid>, (CounterpartyCategoryGroup, CategoryInnerMap)>;

#[derive(Debug, Clone)]
pub struct CategorySubGroup {
    pub id: Option<Nulid>,
    pub name: String,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub operations: Vec<SearchOperation>,
}

#[derive(Debug, Clone)]
pub struct CounterpartyCategoryGroup {
    pub id: Option<Nulid>,
    pub name: String,
    pub kind: String,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub categories: Vec<CategorySubGroup>,
}

impl SearchOperationList {
    pub fn group_by_counterparty_category(
        &self,
        counterparties: &CounterpartyList,
        categories: &CategoryList,
    ) -> Vec<CounterpartyCategoryGroup> {
        let mut outer: CounterpartyOuterMap = HashMap::new();

        for item in self.active_items() {
            let op = &item.operation;
            let cp_id = op.context.counterparty_id;
            let cg_id = op.context.category_id;

            let (cp_group, inner_map) = outer.entry(cp_id).or_insert_with(|| {
                let (name, kind) = resolve_counterparty_name_kind(cp_id, counterparties);
                (
                    CounterpartyCategoryGroup {
                        id: cp_id,
                        name,
                        kind,
                        total_debit: Decimal::ZERO,
                        total_credit: Decimal::ZERO,
                        categories: Vec::new(),
                    },
                    HashMap::new(),
                )
            });

            accumulate_flow(op, &mut cp_group.total_debit, &mut cp_group.total_credit, &mut None);

            let sub = inner_map.entry(cg_id).or_insert_with(|| CategorySubGroup {
                id: cg_id,
                name: resolve_category_name(cg_id, categories),
                total_debit: Decimal::ZERO,
                total_credit: Decimal::ZERO,
                operations: Vec::new(),
            });

            accumulate_flow(op, &mut sub.total_debit, &mut sub.total_credit, &mut None);
            sub.operations.push(item.clone());
        }

        let mut result: Vec<CounterpartyCategoryGroup> = outer
            .into_values()
            .map(|(mut cp_group, inner_map)| {
                let mut categories: Vec<CategorySubGroup> = inner_map
                    .into_values()
                    .map(|mut sub| {
                        sub.operations.sort_by_key(|so| so.operation.date);
                        sub
                    })
                    .collect();
                categories.sort_by(|a, b| a.name.cmp(&b.name));
                cp_group.categories = categories;
                cp_group
            })
            .collect();

        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }
}

/*==============================================================================*/
/*                              SEARCH BUILDER                                   */
/*==============================================================================*/

#[derive(Debug, Default, Copy, Clone)]
pub enum NulidSearchFilter {
    #[default]
    Any,
    NoneOnly,
    One(Nulid),
}

#[derive(Debug, Default, Clone, Builder)]
#[builder(default, build_fn(private, name = "fallible_build"))]
pub struct SearchParams {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub text: Option<String>,
    pub kind: Option<String>,
    pub flow: Option<String>,
    pub counterparty: NulidSearchFilter,
    pub category: NulidSearchFilter,
    pub amount_min: Option<Decimal>,
    pub amount_max: Option<Decimal>,
    pub latest: Option<usize>,
}

impl SearchParamsBuilder {
    pub fn build(&self) -> Result<SearchParams, SearchError> {
        self.fallible_build()
            .map_err(|e| SearchError::SearchParametersBuilder(e.to_string()))
    }
}

/*==============================================================================*/
/*                              SEARCH                                          */
/*==============================================================================*/

/// Checks whether a transaction falls within the date range.
fn matches_date_range(op: &Operation, from: Option<NaiveDate>, to: Option<NaiveDate>) -> bool {
    if let Some(start) = from
        && op.date < start
    {
        return false;
    }
    if let Some(end) = to
        && op.date > end
    {
        return false;
    }
    true
}

/// Checks whether an operation matches the search text.
fn matches_text(op: &Operation, text: Option<&str>) -> bool {
    text.is_none_or(|t| op.description.to_lowercase().contains(t))
}

pub fn search<T: OperationContainer>(container: &T, params: &SearchParams) -> Result<SearchOperationList, SearchError> {
    let mut ops_map = container.get_operations_with_balance();
    ops_map.sort_by_key(|item| item.operation.date);

    let from = params.from;
    let to = params.to;
    let text = params.text.as_deref();
    let kind = params.kind.as_deref();
    let flow = params.flow.as_deref();
    let counterparty = params.counterparty;
    let category = params.category;
    let amount_min = params.amount_min;
    let amount_max = params.amount_max;
    let latest = params.latest;

    if let (Some(start), Some(end)) = (from, to)
        && end < start
    {
        return Err(SearchError::InvalidDate(format!(
            "Invalid date range: end date ({}) is before start date ({})",
            end, start
        )));
    }

    let o_flow_filter = match flow {
        Some(s) => match OperationFlow::try_from(s) {
            Ok(v) => Some(v),
            Err(e) => return Err(SearchError::InvalidData(e.to_string())),
        },
        None => None,
    };

    let o_kind_filter = match kind {
        Some(s) => match OperationKind::try_from(s) {
            Ok(v) => Some(v),
            Err(e) => return Err(SearchError::InvalidData(e.to_string())),
        },
        None => None,
    };

    let mut matched = SearchOperationList::new(params);
    let mut cur_bal = Decimal::ZERO;

    for item in ops_map {
        let op = &item.operation;

        // Filter by date
        if !matches_date_range(op, from, to) {
            continue;
        }
        // Filter by text
        if !matches_text(op, text) {
            continue;
        }

        // Filter by counterpartie
        match &counterparty {
            NulidSearchFilter::Any => {}
            NulidSearchFilter::NoneOnly => {
                if op.context.counterparty_id.is_some() {
                    continue;
                }
            }
            NulidSearchFilter::One(id) => {
                if op.context.counterparty_id != Some(*id) {
                    continue;
                }
            }
        }

        // Filter by catégorie
        match &category {
            NulidSearchFilter::Any => {}
            NulidSearchFilter::NoneOnly => {
                if op.context.category_id.is_some() {
                    continue;
                }
            }
            NulidSearchFilter::One(id) => {
                if op.context.category_id != Some(*id) {
                    continue;
                }
            }
        }

        if let Some(f_op) = o_flow_filter
            && op.flow != f_op
        {
            continue;
        }
        if let Some(k_op) = o_kind_filter
            && op.kind != k_op
        {
            continue;
        }
        if let Some(min) = amount_min
            && op.amount < min
        {
            continue;
        }
        if let Some(max) = amount_max
            && op.amount > max
        {
            continue;
        }

        let mut item = item;
        item.balance = cur_bal + item.operation.flow.to_sign() * item.operation.amount;
        cur_bal = item.balance;
        matched.items.push(item);
    }

    let result = if let Some(n) = latest {
        if matched.items.len() <= n {
            matched
        } else {
            SearchOperationList {
                items: matched.last_n(n),
                params: matched.params,
            }
        }
    } else {
        matched
    };

    Ok(result)
}
