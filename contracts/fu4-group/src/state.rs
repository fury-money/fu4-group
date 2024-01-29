use cosmwasm_std::{Addr, ReadonlyStorage, StdResult, Storage, Snapshot};
use cw_controllers::{Admin, Hooks};
use cw_storage_plus::{Item, Bucket, Map, Strategy};

use tg4::{MemberInfo, TOTAL_KEY};

pub const ADMIN: Admin = Admin::new("admin");
pub const HOOKS: Hooks = Hooks::new("tg4-hooks");

pub const TOTAL: Item<u64> = Item::new(TOTAL_KEY);

pub const MEMBERS: Bucket<Addr, MemberInfo> = Bucket::new(tg4::MEMBERS_KEY);

impl ReadonlyStorage for Snapshot {
    fn get(&self, key: &[u8]) -> StdResult<Option<Vec<u8>>> {
        ReadonlyStorage::get(self, key)
    }

    fn range(&self, start: &[u8], end: &[u8]) -> StdResult<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>>> {
        ReadonlyStorage::range(self, start, end)
    }

    fn prefix(&self, prefix: &[u8]) -> StdResult<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>>> {
        ReadonlyStorage::prefix(self, prefix)
    }
}
