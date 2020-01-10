#![feature(associated_type_defaults)]

mod ca_permissioned;
mod group;
mod permissionless;
mod vote_permissioned;

pub use self::permissionless::PermissionlessGroup;
