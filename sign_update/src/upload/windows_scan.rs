#[cfg(windows)]
mod platform {
    use std::ffi::c_void;

    #[repr(C)]
    struct Guid {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    #[repr(C)]
    struct WlanInterfaceInfo {
        interface_guid: Guid,
        description: [u16; 256],
        state: i32,
    }

    #[repr(C)]
    struct WlanInterfaceInfoList {
        item_count: u32,
        current_index: u32,
        items: [WlanInterfaceInfo; 1],
    }

    #[link(name = "wlanapi")]
    unsafe extern "system" {
        fn WlanOpenHandle(
            client_version: u32,
            reserved: *mut c_void,
            negotiated_version: *mut u32,
            client_handle: *mut *mut c_void,
        ) -> u32;
        fn WlanEnumInterfaces(
            client_handle: *mut c_void,
            reserved: *mut c_void,
            interface_list: *mut *mut WlanInterfaceInfoList,
        ) -> u32;
        fn WlanScan(
            client_handle: *mut c_void,
            interface_guid: *const Guid,
            ssid: *const c_void,
            information_element_data: *const c_void,
            reserved: *mut c_void,
        ) -> u32;
        fn WlanFreeMemory(memory: *mut c_void);
        fn WlanCloseHandle(client_handle: *mut c_void, reserved: *mut c_void) -> u32;
    }

    pub fn request_wlan_scan() -> bool {
        unsafe {
            let mut negotiated_version = 0;
            let mut client_handle = std::ptr::null_mut();
            if WlanOpenHandle(
                2,
                std::ptr::null_mut(),
                &mut negotiated_version,
                &mut client_handle,
            ) != 0
            {
                return false;
            }

            let mut interface_list = std::ptr::null_mut();
            if WlanEnumInterfaces(client_handle, std::ptr::null_mut(), &mut interface_list) != 0 {
                WlanCloseHandle(client_handle, std::ptr::null_mut());
                return false;
            }

            let list = &*interface_list;
            let mut requested = false;
            for index in 0..list.item_count as usize {
                let interface = list.items.as_ptr().add(index);
                requested |= WlanScan(
                    client_handle,
                    &(*interface).interface_guid,
                    std::ptr::null(),
                    std::ptr::null(),
                    std::ptr::null_mut(),
                ) == 0;
            }

            WlanFreeMemory(interface_list.cast());
            WlanCloseHandle(client_handle, std::ptr::null_mut());
            requested
        }
    }
}

#[cfg(windows)]
pub use platform::request_wlan_scan;

#[cfg(not(windows))]
pub fn request_wlan_scan() -> bool {
    false
}
