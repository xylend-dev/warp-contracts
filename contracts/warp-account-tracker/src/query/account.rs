use cosmwasm_std::{Deps, Order, StdResult};
use cw_storage_plus::{Bound, PrefixBound};

use crate::state::{CONFIG, FREE_JOB_ACCOUNTS, FUNDING_ACCOUNTS_BY_USER, TAKEN_JOB_ACCOUNTS};

use account_tracker::{
    Account, AccountResponse, AccountsResponse, ConfigResponse, FundingAccountResponse,
    FundingAccountsResponse, QueryFirstFreeFundingAccountMsg, QueryFirstFreeJobAccountMsg,
    QueryFreeJobAccountsMsg, QueryFundingAccountMsg, QueryFundingAccountsMsg,
    QueryTakenJobAccountsMsg,
};

const QUERY_LIMIT: u32 = 50;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

pub fn query_first_free_job_account(
    deps: Deps,
    data: QueryFirstFreeJobAccountMsg,
) -> StdResult<AccountResponse> {
    let account_owner_ref = &deps.api.addr_validate(data.account_owner_addr.as_str())?;
    let maybe_free_job_account = FREE_JOB_ACCOUNTS
        .prefix_range(
            deps.storage,
            Some(PrefixBound::inclusive(account_owner_ref)),
            Some(PrefixBound::inclusive(account_owner_ref)),
            Order::Ascending,
        )
        .next();
    let free_job_account = match maybe_free_job_account {
        Some(Ok((account, last_job_id))) => Some(Account {
            addr: account.1,
            taken_by_job_id: Some(last_job_id),
        }),
        _ => None,
    };
    Ok(AccountResponse {
        account: free_job_account,
    })
}

pub fn query_taken_job_accounts(
    deps: Deps,
    data: QueryTakenJobAccountsMsg,
) -> StdResult<AccountsResponse> {
    let account_owner_ref = &deps.api.addr_validate(data.account_owner_addr.as_str())?;
    let iter = match data.start_after {
        Some(start_after) => {
            let start_after_account_addr = &deps.api.addr_validate(start_after.as_str())?;
            TAKEN_JOB_ACCOUNTS.range(
                deps.storage,
                Some(Bound::exclusive((
                    account_owner_ref,
                    start_after_account_addr,
                ))),
                None,
                Order::Descending,
            )
        }
        None => TAKEN_JOB_ACCOUNTS.prefix_range(
            deps.storage,
            Some(PrefixBound::inclusive(account_owner_ref)),
            Some(PrefixBound::inclusive(account_owner_ref)),
            Order::Descending,
        ),
    };
    let accounts = iter
        .take(data.limit.unwrap_or(QUERY_LIMIT) as usize)
        .map(|item| {
            item.map(|(account, job_id)| Account {
                addr: account.1,
                taken_by_job_id: Some(job_id),
            })
        })
        .collect::<StdResult<Vec<Account>>>()?;
    Ok(AccountsResponse {
        total_count: accounts.len() as u32,
        accounts,
    })
}

pub fn query_free_job_accounts(
    deps: Deps,
    data: QueryFreeJobAccountsMsg,
) -> StdResult<AccountsResponse> {
    let account_owner_ref = &deps.api.addr_validate(data.account_owner_addr.as_str())?;
    let iter = match data.start_after {
        Some(start_after) => {
            let start_after_account_addr = &deps.api.addr_validate(start_after.as_str())?;
            FREE_JOB_ACCOUNTS.range(
                deps.storage,
                Some(Bound::exclusive((
                    account_owner_ref,
                    start_after_account_addr,
                ))),
                None,
                Order::Descending,
            )
        }
        None => FREE_JOB_ACCOUNTS.prefix_range(
            deps.storage,
            Some(PrefixBound::inclusive(account_owner_ref)),
            Some(PrefixBound::inclusive(account_owner_ref)),
            Order::Descending,
        ),
    };
    let accounts = iter
        .take(data.limit.unwrap_or(QUERY_LIMIT) as usize)
        .map(|item| {
            item.map(|(account, last_job_id)| Account {
                addr: account.1,
                taken_by_job_id: Some(last_job_id),
            })
        })
        .collect::<StdResult<Vec<Account>>>()?;
    Ok(AccountsResponse {
        total_count: accounts.len() as u32,
        accounts,
    })
}

// funding accounts

pub fn query_funding_account(
    deps: Deps,
    data: QueryFundingAccountMsg,
) -> StdResult<FundingAccountResponse> {
    let account_addr_ref = deps.api.addr_validate(data.account_addr.as_str())?;
    let account_owner_addr_ref = deps.api.addr_validate(data.account_owner_addr.as_str())?;

    let funding_accounts = FUNDING_ACCOUNTS_BY_USER.load(deps.storage, &account_owner_addr_ref)?;

    Ok(FundingAccountResponse {
        funding_account: funding_accounts
            .iter()
            .find(|fa| fa.account_addr == account_addr_ref.clone())
            .cloned(),
    })
}

pub fn query_funding_accounts(
    deps: Deps,
    data: QueryFundingAccountsMsg,
) -> StdResult<FundingAccountsResponse> {
    let account_owner_addr_ref = deps.api.addr_validate(data.account_owner_addr.as_str())?;

    let resp = FUNDING_ACCOUNTS_BY_USER.load(deps.storage, &account_owner_addr_ref);

    let funding_accounts = match resp {
        Ok(funding_accounts) => funding_accounts,
        Err(_) => vec![],
    };

    Ok(FundingAccountsResponse { funding_accounts })
}

pub fn query_first_free_funding_account(
    deps: Deps,
    data: QueryFirstFreeFundingAccountMsg,
) -> StdResult<FundingAccountResponse> {
    let account_owner_addr_ref = deps.api.addr_validate(data.account_owner_addr.as_str())?;

    let resp = FUNDING_ACCOUNTS_BY_USER.load(deps.storage, &account_owner_addr_ref);

    let funding_account = match resp {
        Ok(funding_accounts) => funding_accounts
            .iter()
            .find(|fa| fa.taken_by_job_ids.is_empty())
            .cloned(),
        Err(_) => None,
    };

    Ok(FundingAccountResponse { funding_account })
}
