use cosmwasm_std::{Addr, Singleton, Bucket, Snapshot, ReadonlyStorage, StdResult};
use cw_controllers::{Admin, Hooks};
use tg4::{MemberInfo, TOTAL_KEY};

pub const ADMIN: Admin = Admin::new("admin");
pub const HOOKS: Hooks = Hooks::new("tg4-hooks");

pub const TOTAL: Singleton<u64> = Singleton::new(TOTAL_KEY);

pub const MEMBERS: Bucket<Addr, MemberInfo> = Bucket::new(tg4::MEMBERS_KEY);

// Optional: If you need changelog functionality
// pub const MEMBERS_CHANGELOG: Changelog = Changelog::new("members_changelog");

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

// Uncomment the code below if you need to use `Changelog` (MEMBERS_CHANGELOG)

// use cosmwasm_std::Changelog;
// pub const MEMBERS_CHANGELOG: Changelog = Changelog::new("members_changelog");
