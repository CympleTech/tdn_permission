#![feature(associated_type_defaults)]

mod ca_permissioned;
mod permissionless;
mod vote_permissioned;

pub use ca_permissioned::CAPermissionedGroup;
pub use permissionless::PermissionlessGroup;
