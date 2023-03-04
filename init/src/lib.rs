#![no_std]
#![feature(concat_bytes)]
#![feature(maybe_uninit_slice)]

#[macro_use]
extern crate klib;

pub mod init;
mod syscall;
