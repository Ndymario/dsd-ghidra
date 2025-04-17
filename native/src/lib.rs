mod list;
mod loader;
mod sync;
mod traits;
pub mod types;

use core::slice;
use std::{
    ffi::{c_char, CStr},
    fmt::Display,
};

use ds_rom::rom::{raw, Rom};
use loader::{DsRomLoaderData, SafeDsRomLoaderData};
use sync::{DsdSyncData, SafeDsdConfigData};
use traits::{TryIntoSafe, TryIntoUnsafe};

/// # Safety
///
/// * `bytes` and `length` must refer to accessible memory.
#[no_mangle]
pub unsafe extern "C" fn is_valid_ds_rom(bytes: *const u8, length: u32) -> bool {
    let bytes = slice::from_raw_parts(bytes, length as usize);
    let raw_rom = raw::Rom::new(bytes);
    Rom::extract(&raw_rom).is_ok()
}

/// # Safety
///
/// * `bytes` and `length` must refer to accessible memory.
/// * `data` must be allocated and uninitialized.
#[no_mangle]
pub unsafe extern "C" fn get_loader_data(bytes: *const u8, length: u32, data: *mut DsRomLoaderData) -> bool {
    let bytes = slice::from_raw_parts(bytes, length as usize);
    let raw_rom = raw::Rom::new(bytes);
    let Ok(rom) = Rom::extract(&raw_rom) else {
        return false;
    };

    let Some(safe_data) = unwrap_or_log(SafeDsRomLoaderData::new(&rom)) else {
        return false;
    };
    let Some(unsafe_data) = unwrap_or_log(safe_data.try_into_unsafe()) else {
        return false;
    };
    *data = unsafe_data;

    true
}

/// # Safety
///
/// * `data` must be generated by [`get_loader_data`].
/// * This function must be called only once for each [`DsRomLoaderData`] created by [`get_loader_data`].
#[no_mangle]
pub unsafe extern "C" fn free_loader_data(data: *mut DsRomLoaderData) {
    let Some(data) = data.as_ref().cloned() else {
        return;
    };
    let Some(safe_data) = unwrap_or_log(data.try_into_safe()) else {
        return;
    };
    drop(safe_data);
}

/// # Safety
///
/// * `config_path` must be a valid null-terminated string.
/// * `data` must be allocated and uninitialized.
#[no_mangle]
pub unsafe extern "C" fn get_dsd_sync_data(config_path: *const c_char, data: *mut DsdSyncData) -> bool {
    let cstr = CStr::from_ptr(config_path);
    let Some(config_path) = unwrap_or_log(cstr.to_str()) else {
        return false;
    };

    let Some(safe_data) = unwrap_or_log(SafeDsdConfigData::from_config(config_path)) else {
        return false;
    };
    let Some(unsafe_data) = unwrap_or_log(safe_data.try_into_unsafe()) else {
        return false;
    };
    *data = unsafe_data;

    true
}

/// # Safety
///
/// * `data` must be generated by [`get_dsd_sync_data`].
/// * This function must be called only once for each [`DsdSyncData`] created by [`get_dsd_sync_data`].
#[no_mangle]
pub unsafe extern "C" fn free_dsd_sync_data(data: *mut DsdSyncData) {
    let Some(data) = data.as_ref().cloned() else {
        return;
    };
    let Some(safe_data) = unwrap_or_log(data.try_into_safe()) else {
        return;
    };
    drop(safe_data);
}

fn unwrap_or_log<T, E>(result: Result<T, E>) -> Option<T>
where
    E: Display,
{
    match result {
        Ok(data) => Some(data),
        Err(e) => {
            eprintln!("{e}");
            None
        }
    }
}
