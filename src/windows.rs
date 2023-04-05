use core::convert::TryInto;
use std::{ffi::OsString, os::windows::ffi::OsStringExt, slice};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::NetworkManagement::IpHelper::{
    GetAdaptersAddresses, GET_ADAPTERS_ADDRESSES_FLAGS, IP_ADAPTER_ADDRESSES_LH,
};
use windows::Win32::Networking::WinSock::AF_UNSPEC;

use crate::MacAddressError;

const GAA_FLAG_NONE: GET_ADAPTERS_ADDRESSES_FLAGS = GET_ADAPTERS_ADDRESSES_FLAGS(0x0000);

/// Uses bindings to the `Iphlpapi.h` Windows header to fetch the interface devices
/// list with [GetAdaptersAddresses][https://msdn.microsoft.com/en-us/library/windows/desktop/aa365915(v=vs.85).aspx]
/// then loops over the returned list until it finds a network device with a MAC address,
/// and returns it.
///
/// If it fails to find a device, it returns a `NoDevicesFound` error.
pub fn get_mac(name: Option<&str>) -> Result<Option<[u8; 6]>, MacAddressError> {
    let mut adapters = get_adapters()?;
    // Pointer to the current location in the linked list
    let mut ptr = adapters.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

    loop {
        // Break if we've gone through all devices
        if ptr.is_null() {
            break;
        }

        let bytes = unsafe { convert_mac_bytes(ptr) };

        if let Some(name) = name {
            let adapter_name = unsafe { construct_string((*ptr).FriendlyName.as_ptr()) };

            if adapter_name == name {
                return Ok(Some(bytes));
            }
        } else if bytes.iter().any(|&x| x != 0) {
            return Ok(Some(bytes));
        }

        // Otherwise go to the next device
        ptr = unsafe { (*ptr).Next };
    }

    Ok(None)
}

/// Uses bindings to the `Iphlpapi.h` Windows header to fetch the interface devices
/// list with [GetAdaptersAddresses][https://msdn.microsoft.com/en-us/library/windows/desktop/aa365915(v=vs.85).aspx]
/// then loops over the returned list and filters network devices with a MAC address.
///
/// If it fails to find a device, it returns a `NoDevicesFound` error.
pub fn get_mac_list() -> Result<Vec<[u8; 6]>, MacAddressError> {
    let mut adapters = get_adapters()?;
    // Pointer to the current location in the linked list
    let mut ptr = adapters.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

    let mut result = vec![];

    loop {
        // Break if we've gone through all devices
        if ptr.is_null() {
            break;
        }

        let bytes = unsafe { convert_mac_bytes(ptr) };

        if bytes.iter().any(|&x| x != 0) {
            result.push(bytes);
        }

        // Otherwise go to the next device
        ptr = unsafe { (*ptr).Next };
    }

    Ok(result)
}

pub fn get_ifname(mac: &[u8; 6]) -> Result<Option<String>, MacAddressError> {
    let mut adapters = get_adapters()?;
    // Pointer to the current location in the linked list
    let mut ptr = adapters.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

    loop {
        // Break if we've gone through all devices
        if ptr.is_null() {
            break;
        }

        let bytes = unsafe { convert_mac_bytes(ptr) };

        if &bytes == mac {
            let adapter_name = unsafe { construct_string((*ptr).FriendlyName.as_ptr()) };
            let adapter_name = adapter_name
                .into_string()
                .map_err(|_| MacAddressError::InternalError)?;
            return Ok(Some(adapter_name));
        }

        // Otherwise go to the next device
        ptr = unsafe { (*ptr).Next };
    }

    Ok(None)
}

/// Copy over the 6 MAC address bytes to the buffer.
pub(crate) unsafe fn convert_mac_bytes(ptr: *mut IP_ADAPTER_ADDRESSES_LH) -> [u8; 6] {
    ((*ptr).PhysicalAddress)[..6].try_into().unwrap()
}

pub(crate) fn get_adapters() -> Result<Vec<u8>, MacAddressError> {
    let mut buf_len = 0;

    // This will get the number of bytes we need to allocate for all devices
    unsafe {
        GetAdaptersAddresses(AF_UNSPEC.0 as u32, GAA_FLAG_NONE, None, None, &mut buf_len);
    }

    // Allocate `buf_len` bytes, and create a raw pointer to it
    let mut adapters_list = vec![0u8; buf_len as usize];
    let adapter_addresses: *mut IP_ADAPTER_ADDRESSES_LH = adapters_list.as_mut_ptr() as *mut _;

    // Get our list of adapters
    let result = unsafe {
        GetAdaptersAddresses(
            // [IN] Family
            AF_UNSPEC.0 as u32,
            // [IN] Flags
            GAA_FLAG_NONE,
            // [IN] Reserved
            None,
            // [INOUT] AdapterAddresses
            Some(adapter_addresses as *mut _),
            // [INOUT] SizePointer
            &mut buf_len,
        )
    };

    // Make sure we were successful
    if result != ERROR_SUCCESS.0 {
        return Err(MacAddressError::InternalError);
    }

    Ok(adapters_list)
}

unsafe fn construct_string(ptr: *mut u16) -> OsString {
    let slice = slice::from_raw_parts(ptr, get_null_position(ptr));
    OsStringExt::from_wide(slice)
}

unsafe fn get_null_position(ptr: *mut u16) -> usize {
    assert!(!ptr.is_null());

    for i in 0.. {
        if *ptr.offset(i) == 0 {
            return i as usize;
        }
    }

    unreachable!()
}
