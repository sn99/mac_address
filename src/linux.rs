#![allow(dead_code)]

use crate::MacAddressError;
use nix::ifaddrs::*;

/// Uses the `getifaddrs` call to retrieve a list of network interfaces on the
/// host device and returns the first MAC address listed that isn't
/// local-loopback or if a name was specified, that name.
pub fn get_mac(name: Option<&str>) -> Result<Option<[u8; 6]>, MacAddressError> {
    let ifiter = getifaddrs()?;

    for interface in ifiter {
        if let Some(address) = interface.address {
            if let Some(link) = address.as_link_addr() {
                if let Some(bytes) = link.addr() {
                    if let Some(name) = name {
                        if interface.interface_name == name {
                            return Ok(Some(bytes));
                        }
                    } else if bytes.iter().any(|&x| x != 0) {
                        return Ok(Some(bytes));
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Uses the `getifaddrs` call to retrieve a list of network interfaces on the
/// host device and returns all MAC address listed that aren't
/// local-loopback.
pub fn get_mac_list() -> Result<Vec<[u8; 6]>, MacAddressError> {
    let mut result = vec![];

    for interface in getifaddrs()? {
        if let Some(address) = interface.address {
            if let Some(link) = address.as_link_addr() {
                if let Some(bytes) = link.addr() {
                    if bytes.iter().any(|&x| x != 0) {
                        result.push(bytes);
                    }
                }
            }
        }
    }
    Ok(result)
}

pub fn get_ifname(mac: &[u8; 6]) -> Result<Option<String>, MacAddressError> {
    let ifiter = getifaddrs()?;

    for interface in ifiter {
        if let Some(address) = interface.address {
            if let Some(link) = address.as_link_addr() {
                if let Some(bytes) = link.addr() {
                    if &bytes == mac {
                        return Ok(Some(interface.interface_name));
                    }
                }
            }
        }
    }

    Ok(None)
}
