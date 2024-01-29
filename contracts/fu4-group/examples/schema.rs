use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_std::testing::{remove_schemas, Schema, schema_for};
use cosmwasm_std::{export_schema, export_schema_with_title};

pub use tg4::{AdminResponse, MemberListResponse, MemberResponse, TotalPointsResponse};
pub use tg4_group::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema_with_title::<InstantiateMsg>(&out_dir, "InstantiateMsg");
    export_schema_with_title::<ExecuteMsg>(&out_dir, "ExecuteMsg");
    export_schema_with_title::<QueryMsg>(&out_dir, "QueryMsg");
    export_schema::<AdminResponse>(&out_dir);
    export_schema::<MemberListResponse>(&out_dir);
    export_schema::<MemberResponse>(&out_dir);
    export_schema::<TotalPointsResponse>(&out_dir);
}
