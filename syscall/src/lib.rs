#![no_std]
#![feature(concat_bytes)]
#![feature(maybe_uninit_slice)]
#![feature(asm_const)]
#![feature(generators, generator_trait)]
#![feature(core_intrinsics)]

extern crate klib;

#[macro_use]
pub mod macros;

pub mod error;
pub mod payload;
pub mod syscall;
