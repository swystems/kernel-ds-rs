// SPDX-License-Identifier: GPL-2.0

//! Rust minimal sample.
mod vallocator;

use kernel::prelude::*;
use kernel::bindings;
use crate::vallocator::VAllocator;

module! {
    type: RustMinimal,
    name: "rustdev",
    author: "Rust for Linux Contributors",
    description: "Rust minimal sample",
    license: "GPL",
}

struct RustMinimal {
    vec_in_global: Vec<i32>,
    vec_in_valloc: Vec<i32, VAllocator>,
}

impl kernel::Module for RustMinimal {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust minimal sample (init)\n");

        // array larger than 4m is ok because it is on the stack
        let _ = [0u8; 1024*1024*1024+1];

        // vec try to apply 4m from `Global` allocator
        // it will be success
        {
            pr_info!("let vec_4k = Vec::try_with_capacity(1024 * 1024)\n");
            let _vec_4k: Vec<i32> = Vec::try_with_capacity(1024 * 1024).unwrap();
        }
        pr_info!("vec_4k dropped\n");

        // vec try to apply 4m+4b from `Global` allocator
        // it will failed but won't panic
        pr_info!("let vec_in_global = Vec::try_with_capacity(1024 * 1024 + 1)\n");
        let vec_in_global = Vec::try_with_capacity(1024 * 1024 + 1)
            .unwrap_or_else(|_| Vec::try_with_capacity(1024 * 1024).unwrap());

        // vec try to apply 4m+4b from allocator which based on vmalloc_user
        // it will be success
        // but the more memory vec takes up, the more likely it is to be killed by the OOM killer
        let vec_in_valloc = Vec::try_with_capacity_in(1024 * 1024 + 1, VAllocator).expect("???");

        Ok(RustMinimal { vec_in_global, vec_in_valloc })
    }
}

impl Drop for RustMinimal {
    fn drop(&mut self) {
        pr_info!(
            "vec_in_global.capacity() = {}\nvec_in_valloc.capacity() = {}\n",
            self.vec_in_global.capacity(),
            self.vec_in_valloc.capacity()
        );
        pr_info!("Rust minimal sample (exit)\n");
    }
}
