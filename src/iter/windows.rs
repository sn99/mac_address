use crate::os;
use crate::{MacAddress, MacAddressError};
use windows::Win32::NetworkManagement::IpHelper::IP_ADAPTER_ADDRESSES_LH;

/// An iterator over all available MAC addresses on the system.
pub struct MacAddressIterator {
    // So we don't UAF during iteration.
    _buffer: Vec<u8>,
    ptr: *mut IP_ADAPTER_ADDRESSES_LH,
}

impl MacAddressIterator {
    /// Creates a new `MacAddressIterator`.
    pub fn new() -> Result<MacAddressIterator, MacAddressError> {
        let mut adapters = os::get_adapters()?;
        let ptr = adapters
            .as_mut_ptr()
            .cast::<windows::Win32::NetworkManagement::IpHelper::IP_ADAPTER_ADDRESSES_LH>();

        Ok(Self {
            _buffer: adapters,
            ptr,
        })
    }
}

impl Iterator for MacAddressIterator {
    type Item = MacAddress;

    fn next(&mut self) -> Option<MacAddress> {
        if self.ptr.is_null() {
            None
        } else {
            let bytes = unsafe { os::convert_mac_bytes(self.ptr) };

            self.ptr = unsafe { (*self.ptr).Next };

            Some(MacAddress::new(bytes))
        }
    }
}
