use core::{
    cell::LazyCell,
    ffi::{c_char, c_int, c_void},
};

use crate::{binds::*, get_func_ptr_by_name, resolve_func};

#[repr(transparent)]
struct CachedPtr<T, F = fn() -> T>(LazyCell<T, F>);
unsafe impl<T, F> Sync for CachedPtr<T, F> {}

impl<T, F: FnOnce() -> T> CachedPtr<T, F> {
    #[inline]
    pub const fn new(f: F) -> CachedPtr<T, F> {
        CachedPtr(LazyCell::new(f))
    }
}

pub const GetProcAddress_: &str = concat!("GetProcAddress", "\0");
pub const MessageBoxA_: &str = concat!("MessageBoxA", "\0");
pub const GlobalAlloc_: &str = concat!("GlobalAlloc", "\0");

pub type LoadLibraryAFn = unsafe extern "system" fn(lpFileName: *const u8) -> PVOID;
pub type GetProcAddressFn = unsafe extern "system" fn(hmodule: PVOID, name: *const u8) -> PVOID;
pub type MessageBoxAFn =
    unsafe extern "system" fn(h: PVOID, text: LPCSTR, cation: LPCSTR, t: u32) -> u32;
pub type OutputDebugStringAFn = unsafe extern "C" fn(*const i8);
pub type DbgPrintFn = unsafe extern "C" fn(Format: *const u8, ...) -> NTSTATUS;
pub type GetModuleHandleAFn = unsafe extern "system" fn(lpModuleName: LPCSTR) -> PVOID;
pub type CreateFileAFn = unsafe extern "system" fn(
    lpFileName: LPCSTR,
    dwDesiredAccess: u32,
    dwShareMode: u32,
    lpSecurityAttributes: PVOID,
    dwCreationDisposition: u32,
    dwFlagsAndATtributes: u32,
    hTemplateFile: PVOID,
) -> PVOID;
pub type wsprintfaFn = unsafe extern "system" fn(outbuf: LPSTR, inbuf: LPCSTR, ...);
pub type ReadFileFn = unsafe extern "system" fn(
    hFile: PVOID,
    lpBuf: PVOID,
    nNumberOfBytesToRead: u32,
    lpNumberOfBytesRead: *mut u32,
    lpOverlapped: PVOID,
) -> c_int;
pub type GlobalAllocFn = unsafe extern "system" fn(flags: u32, byte_count: usize) -> PVOID;
pub type GlobalFreeFn = unsafe extern "system" fn(addr: PVOID);
pub type VirtualAllocFn = unsafe extern "system" fn(
    lpAddress: *const c_void,
    dwSize: usize,
    flAllocationType: u32,
    flProtect: u32,
) -> PVOID;
pub type VirtualFreeFn =
    unsafe extern "system" fn(lpAddress: PVOID, dwSize: usize, dwFreeType: u32) -> bool;

pub type VirtualProtectFn = unsafe extern "system" fn(
    lpAddress: *const c_void,
    dwSize: usize,
    flNewProtect: u32,
    lpflOldProtect: *mut u32,
) -> c_char;
pub type GetFileSizeFn = unsafe extern "system" fn(hFile: PVOID, lpHighFileSize: *mut u32) -> u32;
pub type CreateThreadFn = unsafe extern "system" fn(
    lpThreadAttributes: *const c_void,
    dwStackSize: usize,
    lpStartAddress: *const c_void,
    lpParameter: *const c_void,
    dwCreationFlags: u32,
    lpThreadId: *mut u32,
) -> *mut c_void;

pub type RtlAddFunctionTableFn = unsafe extern "system" fn(
    FunctionTable: *const c_void,
    EntryCount: u32,
    BaseAddress: u64,
) -> u32;

pub type ExpandEnvironmentStringsAFn =
    unsafe extern "system" fn(lpSrc: *const u8, lpDst: *mut u8, size: u32) -> u32;

// pub fn get_kernel32_test() -> PVOID {
//     static KERNEL32: CachedPtr<PVOID> = CachedPtr::new(|| {
//         let KERNEL32_STR: [u16; 13] = [75, 69, 82, 78, 69, 76, 51, 50, 46, 68, 76, 76, 0];
//         crate::get_module_by_name(KERNEL32_STR.as_ptr())
//     });

//     *KERNEL32.0
// }

pub fn get_kernel32() -> PVOID {
    let KERNEL32_STR: [u16; 13] = [75, 69, 82, 78, 69, 76, 51, 50, 46, 68, 76, 76, 0];
    crate::get_module_by_name(KERNEL32_STR.as_ptr())
}

pub fn get_user32(kernel32_ptr: PVOID) -> PVOID {
    let user32 = concat!("user32.dll", "\0");
    let mut u32_ptr = unsafe { (fetch_load_library(kernel32_ptr))(user32.as_ptr() as *const _) };
    if u32_ptr.is_null() {
        u32_ptr = unsafe { (fetch_get_module_handle(kernel32_ptr))(user32.as_ptr() as *const i8) };
    }

    u32_ptr
}

pub fn fetch_wsprintf(user32_ptr: PVOID) -> wsprintfaFn {
    resolve_func!(user32_ptr, "wsprintfA")
}

pub fn fetch_expand_environment_strings(kernel32_ptr: PVOID) -> ExpandEnvironmentStringsAFn {
    resolve_func!(kernel32_ptr, "ExpandEnvironmentStringsA")
}

pub fn fetch_rtl_add_fn_table(kernel32_ptr: PVOID) -> RtlAddFunctionTableFn {
    resolve_func!(kernel32_ptr, "RtlAddFunctionTable")
}

pub fn fetch_create_thread(kernel32_ptr: PVOID) -> CreateThreadFn {
    resolve_func!(kernel32_ptr, "CreateThread")
}

pub fn fetch_global_alloc(kernel32_ptr: PVOID) -> GlobalAllocFn {
    resolve_func!(kernel32_ptr, "GlobalAlloc")
}

pub fn fetch_global_free(kernel32_ptr: PVOID) -> GlobalFreeFn {
    resolve_func!(kernel32_ptr, "GlobalFree")
}

pub fn fetch_get_file_size(kernel32_ptr: PVOID) -> GetFileSizeFn {
    resolve_func!(kernel32_ptr, "GetFileSize")
}

pub fn fetch_output_debug_string(kernel32_ptr: PVOID) -> OutputDebugStringAFn {
    resolve_func!(kernel32_ptr, "OutputDebugStringA")
}

pub fn fetch_get_module_handle(kernel32_ptr: PVOID) -> GetModuleHandleAFn {
    resolve_func!(kernel32_ptr, "GetModuleHandleA")
}

pub fn fetch_get_proc_address(kernel32_ptr: PVOID) -> GetProcAddressFn {
    resolve_func!(kernel32_ptr, "GetProcAddress")
}

pub fn fetch_load_library(kernel32_ptr: PVOID) -> LoadLibraryAFn {
    resolve_func!(kernel32_ptr, "LoadLibraryA")
}

pub fn fetch_create_file(kernel32_ptr: PVOID) -> CreateFileAFn {
    resolve_func!(kernel32_ptr, "CreateFileA")
}

pub fn fetch_read_file(kernel32_ptr: PVOID) -> ReadFileFn {
    resolve_func!(kernel32_ptr, "ReadFile")
}

pub fn fetch_virtual_alloc(kernel32_ptr: PVOID) -> VirtualAllocFn {
    resolve_func!(kernel32_ptr, "VirtualAlloc")
}

pub fn fetch_virtual_protect(kernel32_ptr: PVOID) -> VirtualProtectFn {
    resolve_func!(kernel32_ptr, "VirtualProtect")
}
