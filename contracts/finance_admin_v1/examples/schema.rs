use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use finance_admin_v1::msg::{CustomHandleMsg, CustomQueryAnswer, CustomQueryMsg, InitMsg};
use shared_types::finance_admin;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InitMsg), &out_dir);
    export_schema(
        &schema_for!(finance_admin::HandleMsg<CustomHandleMsg>),
        &out_dir,
    );
    export_schema(
        &schema_for!(finance_admin::QueryMsg<CustomQueryMsg>),
        &out_dir,
    );
    export_schema(&schema_for!(finance_admin::QueryAnswer), &out_dir);
    export_schema(&schema_for!(CustomQueryAnswer), &out_dir);
}
