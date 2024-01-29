use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    SubMsg, WasmMsg,
};

use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;

use cw4::{MemberChangedHookMsg, MemberDiff};
use tg4::{Member, MemberInfo, MemberListResponse, MemberResponse, TotalPointsResponse};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ADMIN, HOOKS, MEMBERS, TOTAL};

const CONTRACT_NAME: &str = "crates.io:tg4-group";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    create(deps, msg.admin, msg.members, env.block.height)?;
    Ok(Response::default())
}

pub fn create(
    mut deps: DepsMut,
    admin: Option<String>,
    members: Vec<Member>,
    height: u64,
) -> Result<(), ContractError> {
    let admin_addr = admin
        .map(|admin| deps.api.addr_validate(&admin))
        .transpose()?;
    ADMIN.set(deps.branch(), admin_addr)?;

    let mut total = 0u64;
    for member in members.into_iter() {
        total += member.points;
        let member_addr = deps.api.addr_validate(&member.addr)?;
        MEMBERS.save(
            deps.storage,
            &member_addr,
            &MemberInfo::new(member.points),
            height,
        )?;
    }
    TOTAL.save(deps.storage, &total)?;

    Ok(())
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateAdmin { admin } => Ok(ADMIN.execute_update_admin(
            deps,
            info,
            admin.map(|admin| deps.api.addr_validate(&admin)).transpose()?,
        )?),
        ExecuteMsg::UpdateMembers { add, remove } => {
            execute_update_members(deps, env, info, add, remove)
        }
        ExecuteMsg::AddHook { addr } => {
            Ok(HOOKS.execute_add_hook(&ADMIN, deps, info, deps.api.addr_validate(&addr)?)?)
        }
        ExecuteMsg::RemoveHook { addr } => {
            Ok(HOOKS.execute_remove_hook(&ADMIN, deps, info, deps.api.addr_validate(&addr)?)?)
        }
    }
}

pub fn execute_update_members(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    add: Vec<Member>,
    remove: Vec<String>,
) -> Result<Response, ContractError> {
    let attributes = vec![
        attr("action", "update_members"),
        attr("added", add.len().to_string()),
        attr("removed", remove.len().to_string()),
        attr("sender", &info.sender),
    ];

    let diff = update_members(deps.branch(), env.block.height, info.sender, add, remove)?;
    let messages = HOOKS.prepare_hooks(deps.storage, |h| {
        diff.clone().into_cosmos_msg(h).map(SubMsg::new)
    })?;
    Ok(Response::new()
        .add_submessages(messages)
        .add_attributes(attributes))
}

pub fn update_members(
    mut deps: DepsMut,
    height: u64,
    sender: Addr,
    to_add: Vec<Member>,
    to_remove: Vec<String>,
) -> Result<MemberChangedHookMsg, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &sender)?;

    let mut total = TOTAL.load(deps.storage)?;
    let mut diffs: Vec<MemberDiff> = vec![];

    for add in to_add.into_iter() {
        let add_addr = deps.api.addr_validate(&add.addr)?;
        MEMBERS.update(deps.storage, &add_addr, |old| -> StdResult<_> {
            total -= old.clone().unwrap_or_default().points;
            total += add.points;
            diffs.push(MemberDiff::new(add.addr, old.map(|mi| mi.points), Some(add.points)));
            Ok(MemberInfo::new(add.points))
        })?;
    }

    for remove in to_remove.into_iter() {
        let remove_addr = deps.api.addr_validate(&remove)?;
        let old = MEMBERS.may_load(deps.storage, &remove_addr)?;
        if let Some(member_info) = old {
            diffs.push(MemberDiff::new(remove, Some(member_info.points), None));
            total -= member_info.points;
            MEMBERS.remove(deps.storage, &remove_addr)?;
        }
    }

    TOTAL.save(deps.storage, &total)?;
    Ok(MemberChangedHookMsg { diffs })
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Member {
            addr,
            at_height: height,
        } => to_binary(&query_member(deps, addr, height)?),
        QueryMsg::ListMembers { start_after, limit } => {
            to_binary(&list_members(deps, start_after, limit)?)
        }
        QueryMsg::TotalPoints {} => to_binary(&query_total_points(deps)?),
        QueryMsg::Admin {} => to_binary(&ADMIN.query_admin(deps)?),
        QueryMsg::Hooks {} => to_binary(&HOOKS.query_hooks(deps)?),
    }
}

fn query_total_points(deps: Deps) -> StdResult<TotalPointsResponse> {
    let points = TOTAL.load(deps.storage)?;
    Ok(TotalPointsResponse { points })
}

fn query_member(deps: Deps, addr: String, height: Option<u64>) -> StdResult<MemberResponse> {
    let addr = deps.api.addr_validate(&addr)?;
    let member_info = match height {
        Some(h) => MEMBERS.may_load_at_height(deps.storage, &addr, h),
        None => MEMBERS.may_load(deps.storage, &addr),
    }?;
    Ok(member_info.into())
}

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

fn list_members(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<MemberListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let addr = maybe_addr(deps.api, start_after)?;
    let start = addr.as_ref().map(Bound::exclusive);

    let members = MEMBERS
        .range(deps.storage, start, None)
        .take(limit)
        .map(|item| {
            item.map(|(addr, member_info)| Member {
                addr: addr.into(),
                points: member_info.points,
            })
        })
        .collect::<StdResult<_>>()?;

    Ok(MemberListResponse { members })
}
