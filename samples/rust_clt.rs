// SPDX-License-Identifier: GPL-2.0
//! Rust module for CLT 2023
use kernel::prelude::*;

module! {
    type: RustCltModule,
    name: "rust_clt_module",
    author: "Alexander Böhm",
    description: "Rust Module for CLT 2023",
    license: "GPL v2",
}

struct RustCltModule { name: &'static CStr }

impl kernel::Module for RustCltModule {
    fn init(
        name: &'static CStr,
        _module: &'static ThisModule
    ) -> Result<Self> {
        pr_info!("Hello CLT 2023 from kernel module {name}!");
        Ok(Self { name })
    }
}

impl Drop for RustCltModule {
    fn drop(&mut self) {
        pr_info!("Goodbye from kernel module {}!", self.name);
    }
}