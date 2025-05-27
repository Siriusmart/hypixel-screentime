#![allow(static_mut_refs)]

mod config;
pub use config::Config;

mod storage;
pub use storage::*;

pub mod tasks;

mod web;
pub use web::run;

mod mermaid;
