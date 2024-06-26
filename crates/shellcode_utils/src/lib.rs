#![feature(allocator_api)]
#![feature(extended_varargs_abi_support)]
#![feature(strict_provenance)]
#![allow(dead_code)]
#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(overflowing_literals)]

pub mod alloc;
pub mod binds;
pub mod consts;
pub mod functions;
pub mod prelude;

use crate::binds::*;
use core::arch::asm;
use core::panic::PanicInfo;
use core::ptr::null_mut;

pub fn get_module_by_name(module_name: *const u16) -> PVOID {
    let peb: *mut PEB;
    unsafe {
        asm!(
            "mov {}, gs:[0x60]",
            out(reg) peb,
        );
        let ldr = (*peb).Ldr;
        let list_entry = &((*ldr).InLoadOrderModuleList);
        let mut cur_module: *const LDR_DATA_TABLE_ENTRY = &list_entry as *const _ as *const _;
        loop {
            if cur_module.is_null() || (*cur_module).BaseAddress.is_null() {
                // TODO: when to break
            }
            let cur_name = (*cur_module).BaseDllName.Buffer;
            if !cur_name.is_null() {
                if compare_raw_str(module_name, cur_name) {
                    return (*cur_module).BaseAddress;
                }
            }
            let flink = (*cur_module).InLoadOrderModuleList.Flink;
            cur_module = flink as *const LDR_DATA_TABLE_ENTRY;
        }
    }
}

#[macro_export]
macro_rules! resolve_func {
    ($module:expr, $name:expr) => {
        unsafe {
            core::mem::transmute($crate::get_func_ptr_by_name(
                $module,
                concat!($name, "\0").as_ptr(),
            ))
        }
    };
}

pub fn get_func_ptr_by_name(module: PVOID, func_name: *const u8) -> PVOID {
    let idh: *const IMAGE_DOS_HEADER = module as *const _;
    unsafe {
        if (*idh).e_magic != IMAGE_DOS_SIGNATURE {
            return null_mut();
        }
        let e_lfanew = (*idh).e_lfanew;
        let nt_headers: *const IMAGE_NT_HEADERS =
            (module as *const u8).offset(e_lfanew as isize) as *const _;
        let op_header = &(*nt_headers).OptionalHeader;
        let virtual_addr = (&op_header.DataDirectory[0]).VirtualAddress;
        let export_dir: *const IMAGE_EXPORT_DIRECTORY =
            (module as *const u8).offset(virtual_addr as _) as _;
        let number_of_names = (*export_dir).NumberOfNames;
        let addr_of_funcs = (*export_dir).AddressOfFunctions;
        let addr_of_names = (*export_dir).AddressOfNames;
        let addr_of_ords = (*export_dir).AddressOfNameOrdinals;
        for i in 0..number_of_names {
            let name_rva_p: *const DWORD =
                (module as *const u8).offset((addr_of_names + i * 4) as isize) as *const _;
            let name_index_p: *const WORD =
                (module as *const u8).offset((addr_of_ords + i * 2) as isize) as *const _;
            let name_index = name_index_p.as_ref().unwrap();
            let mut off: u32 = (4 * name_index) as u32;
            off = off + addr_of_funcs;
            let func_rva: *const DWORD = (module as *const u8).offset(off as _) as *const _;

            let name_rva = name_rva_p.as_ref().unwrap();
            let curr_name = (module as *const u8).offset(*name_rva as isize);

            if *curr_name == 0 {
                continue;
            }
            if compare_raw_str(func_name, curr_name) {
                let res = (module as *const u8).offset(*func_rva as isize);
                return res as _;
            }
        }
    }
    return null_mut();
}

unsafe fn u16_ptr_len(ptr: *const u16) -> usize {
    let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
    return len;
}

fn compare_str_u16(s: &str, u: *const u16) -> bool {
    unsafe {
        let len = (0..).take_while(|&i| *u.offset(i) != 0).count();
        let slice = core::slice::from_raw_parts(u, len);
        let s_len = s.len();
        if len != s_len {
            return false;
        }
        let ss = s.as_bytes();
        for i in 0..len {
            if slice[i] != ss[i] as u16 {
                return false;
            }
        }
        return true;
    }
}

// TODO: use alloc
pub fn str_to_u16_ptr(s: &str, buf: &mut [u16]) {
    let s_len = s.len();
    let s_bytes = s.as_bytes();

    // for i in 0..255 {

    // }
    // let buf = vec![0u16;s_len+1];
    for i in 0..s_len {
        buf[i] = s_bytes[i] as _;
    }
    buf[s_len] = 0;
    //    buf.as_ptr()
}

use num_traits::Num;

pub fn compare_raw_str<T>(s: *const T, u: *const T) -> bool
where
    T: Num,
{
    unsafe {
        let u_len = (0..).take_while(|&i| !(*u.offset(i)).is_zero()).count();
        let u_slice = core::slice::from_raw_parts(u, u_len);

        let s_len = (0..).take_while(|&i| !(*s.offset(i)).is_zero()).count();
        let s_slice = core::slice::from_raw_parts(s, s_len);

        if s_len != u_len {
            return false;
        }
        for i in 0..s_len {
            if s_slice[i] != u_slice[i] {
                return false;
            }
        }
        return true;
    }
}

fn compare_str_u8(s: &str, u: *const u8) -> bool {
    unsafe {
        let len = (0..).take_while(|&i| *u.offset(i) != 0).count();
        let slice = core::slice::from_raw_parts(u, len);
        let s_len = s.len();
        if len != s_len {
            return false;
        }
        let ss = s.as_bytes();
        for i in 0..len {
            if slice[i] != ss[i] as u8 {
                return false;
            }
        }
        return true;
    }
}

fn str_to_i8(s: &str) -> *const i8 {
    s.as_bytes().as_ptr() as *const i8
}
