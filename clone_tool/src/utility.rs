use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::ptr;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use winapi::ctypes::c_void;
use winapi::shared::minwindef::DWORD;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::fileapi::{CreateFileW, GetLogicalDrives, ReadFile, WriteFile, OPEN_EXISTING};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::ioapiset::DeviceIoControl;
use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
use winapi::um::winbase::{FILE_FLAG_NO_BUFFERING, FILE_FLAG_SEQUENTIAL_SCAN, FILE_FLAG_WRITE_THROUGH};
use winapi::um::winnt::{
    FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE,
};
use winapi::um::winioctl::{
    DISK_EXTENT, FSCTL_DISMOUNT_VOLUME, FSCTL_LOCK_VOLUME, GET_LENGTH_INFORMATION,
    IOCTL_DISK_GET_LENGTH_INFO, IOCTL_STORAGE_QUERY_PROPERTY, IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS,
    VOLUME_DISK_EXTENTS,
};

// Larger chunks reduce per-call overhead; depth 3 lets read/write threads pipeline (double/triple buffering).
const TRANSFER_CHUNK_SIZE: usize = 8 * 1024 * 1024;
const TRANSFER_QUEUE_DEPTH: usize = 3;
// FILE_FLAG_NO_BUFFERING requires buffers/lengths aligned to the device sector size; USB card
// readers virtually always report 512-byte logical sectors, so this is a safe universal choice.
const SECTOR_ALIGN: usize = 512;

fn round_up_to_sector(len: usize) -> usize {
    (len + SECTOR_ALIGN - 1) / SECTOR_ALIGN * SECTOR_ALIGN
}

/// Page-aligned buffer required for unbuffered (FILE_FLAG_NO_BUFFERING) device I/O.
struct AlignedBuffer {
    ptr: *mut u8,
    len: usize,
}

impl AlignedBuffer {
    fn new(len: usize) -> Self {
        let ptr = unsafe { VirtualAlloc(ptr::null_mut(), len, MEM_COMMIT, PAGE_READWRITE) as *mut u8 };
        assert!(!ptr.is_null(), "VirtualAlloc failed for transfer buffer");
        Self { ptr, len }
    }

    fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        unsafe { VirtualFree(self.ptr as *mut c_void, 0, MEM_RELEASE) };
    }
}

// Sole owner moves between the reader and writer threads; never aliased concurrently.
unsafe impl Send for AlignedBuffer {}

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Copy, Clone)]
struct STORAGE_PROPERTY_QUERY {
    PropertyId: u32,
    QueryType: u32,
    AdditionalParameters: [u8; 1],
}

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Copy, Clone)]
struct STORAGE_DEVICE_DESCRIPTOR {
    Version: u32,
    Size: u32,
    DeviceType: u8,
    DeviceTypeModifier: u8,
    RemovableMedia: u8,
    CommandQueueing: u8,
    VendorIdOffset: u32,
    ProductIdOffset: u32,
    ProductRevisionOffset: u32,
    SerialNumberOffset: u32,
    BusType: u32,
    RawPropertiesLength: u32,
}

#[derive(Debug, Clone)]
pub struct RawDrive {
    pub name: String,
    pub capacity: u64,
}

#[derive(Debug, Clone)]
pub struct TransferProgress {
    pub done: u64,
    pub total: u64,
    pub eta: Option<Duration>,
}

pub fn list_raw_drives() -> Vec<RawDrive> {
    let mut drives = Vec::new();

    for index in 0..16 {
        let device_name = format!("\\\\.\\PhysicalDrive{}", index);

        if !is_removable_physical_drive(&device_name) {
            continue;
        }

        if let Some(capacity) = get_raw_drive_capacity(&device_name) {
            drives.push(RawDrive {
                name: device_name,
                capacity,
            });
        }
    }

    drives
}

fn is_removable_physical_drive(device_name: &str) -> bool {
    let handle = open_device_for_read(device_name);
    if handle == INVALID_HANDLE_VALUE {
        return false;
    }

    let mut query = STORAGE_PROPERTY_QUERY {
        PropertyId: 0,
        QueryType: 0,
        AdditionalParameters: [0],
    };

    let mut buffer: [u8; 256] = [0; 256];
    let mut bytes_returned: DWORD = 0;

    let ok = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_STORAGE_QUERY_PROPERTY,
            &mut query as *mut _ as *mut c_void,
            std::mem::size_of::<STORAGE_PROPERTY_QUERY>() as DWORD,
            buffer.as_mut_ptr() as *mut c_void,
            buffer.len() as DWORD,
            &mut bytes_returned,
            ptr::null_mut(),
        )
    };

    unsafe { CloseHandle(handle) };

    if ok == 0 {
        return false;
    }

    let descriptor = unsafe { &*(buffer.as_ptr() as *const STORAGE_DEVICE_DESCRIPTOR) };
    descriptor.RemovableMedia != 0
}

fn get_raw_drive_capacity(device_name: &str) -> Option<u64> {
    let handle = open_device_for_read(device_name);
    if handle == INVALID_HANDLE_VALUE {
        return None;
    }

    let mut disk_length: GET_LENGTH_INFORMATION = unsafe { std::mem::zeroed() };
    let mut bytes_returned: DWORD = 0;

    let ok = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_DISK_GET_LENGTH_INFO,
            ptr::null_mut(),
            0,
            &mut disk_length as *mut _ as *mut c_void,
            std::mem::size_of::<GET_LENGTH_INFORMATION>() as DWORD,
            &mut bytes_returned,
            ptr::null_mut(),
        )
    };

    unsafe { CloseHandle(handle) };

    if ok == 0 {
        return None;
    }

    Some(*unsafe { disk_length.Length.QuadPart() } as u64)
}

pub fn clone_drive_to_image<F>(
    device_name: &str,
    output_path: &str,
    total_size: u64,
    mut on_progress: F,
) -> Result<(), String>
where
    F: FnMut(TransferProgress),
{
    let handle = open_device_for_read(device_name);
    if handle == INVALID_HANDLE_VALUE {
        return Err(format!("Could not open {}", device_name));
    }

    let file = File::create(output_path).map_err(|e| {
        unsafe { CloseHandle(handle) };
        e.to_string()
    })?;
    if total_size > 0 {
        file.set_len(total_size).map_err(|e| {
            unsafe { CloseHandle(handle) };
            e.to_string()
        })?;
    }

    // Write to the output file on its own thread so device reads never wait on local disk I/O.
    let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(TRANSFER_QUEUE_DEPTH);

    let writer_thread = thread::spawn(move || -> Result<(), String> {
        let mut writer = BufWriter::with_capacity(TRANSFER_CHUNK_SIZE, file);
        for chunk in rx {
            writer.write_all(&chunk).map_err(|e| e.to_string())?;
        }
        writer.flush().map_err(|e| e.to_string())
    });

    let mut buffer = vec![0u8; TRANSFER_CHUNK_SIZE];
    let mut copied = 0u64;
    let start_time = Instant::now();

    loop {
        let mut bytes_read: DWORD = 0;
        let ok = unsafe {
            ReadFile(
                handle,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len() as DWORD,
                &mut bytes_read,
                ptr::null_mut(),
            )
        };

        if ok == 0 || bytes_read == 0 {
            break;
        }

        if tx.send(buffer[..bytes_read as usize].to_vec()).is_err() {
            break;
        }
        copied += bytes_read as u64;

        let eta = estimate_remaining_time(start_time.elapsed(), copied, total_size);
        on_progress(TransferProgress {
            done: copied,
            total: total_size,
            eta,
        });
    }

    drop(tx);
    unsafe { CloseHandle(handle) };

    writer_thread.join().map_err(|_| "Writer thread panicked.".to_string())??;

    on_progress(TransferProgress {
        done: copied,
        total: total_size,
        eta: None,
    });

    Ok(())
}

pub fn write_image_to_drive<F>(
    input_path: &str,
    device_name: &str,
    drive_capacity: u64,
    mut on_progress: F,
) -> Result<(), String>
where
    F: FnMut(TransferProgress),
{
    let file = File::open(input_path).map_err(|e| e.to_string())?;
    let image_size = file.metadata().map_err(|e| e.to_string())?.len();

    if image_size == 0 {
        return Err("Input image is empty.".to_string());
    }

    if drive_capacity > 0 && image_size > drive_capacity {
        return Err(format!(
            "Image is larger than target drive: {} > {}",
            format_capacity(image_size),
            format_capacity(drive_capacity)
        ));
    }

    let disk_index = parse_physical_drive_index(device_name)
        .ok_or_else(|| format!("Could not parse physical drive index from {}", device_name))?;

    let locked_volume_handles = lock_and_dismount_volumes_for_disk(disk_index)?;

    let handle = open_device_for_write(device_name);
    if handle == INVALID_HANDLE_VALUE {
        for volume_handle in locked_volume_handles {
            unsafe { CloseHandle(volume_handle) };
        }
        return Err(format!("Could not open {} for writing", device_name));
    }

    // Read the source image on its own thread so the next chunk is ready as soon as the
    // current WriteFile call completes, keeping the device's write pipe continuously fed.
    // Buffers are page-aligned and lengths sector-rounded because the write handle is opened
    // with FILE_FLAG_NO_BUFFERING, which bypasses the system cache entirely so throughput
    // reflects the card's real, sustained speed instead of bursting into RAM then stalling.
    let (tx, rx) = mpsc::sync_channel::<Result<(AlignedBuffer, usize, usize), String>>(TRANSFER_QUEUE_DEPTH);

    let reader_thread = thread::spawn(move || {
        let mut reader = BufReader::with_capacity(TRANSFER_CHUNK_SIZE, file);
        loop {
            let mut buffer = AlignedBuffer::new(TRANSFER_CHUNK_SIZE);
            match reader.read(buffer.as_mut_slice()) {
                Ok(0) => break,
                Ok(n) => {
                    let aligned_len = round_up_to_sector(n);
                    let is_last = n < TRANSFER_CHUNK_SIZE;
                    if tx.send(Ok((buffer, n, aligned_len))).is_err() || is_last {
                        break;
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                    break;
                }
            }
        }
    });

    let mut written_total = 0u64; // aligned bytes actually placed on the disk
    let mut logical_done = 0u64; // real image bytes copied, used for progress/eta
    let start_time = Instant::now();
    let mut failure: Option<String> = None;

    for msg in rx {
        let (buffer, logical_len, mut aligned_len) = match msg {
            Ok(v) => v,
            Err(e) => {
                failure = Some(e);
                break;
            }
        };

        if drive_capacity > 0 {
            let remaining = drive_capacity.saturating_sub(written_total);
            aligned_len = aligned_len.min(remaining as usize);
        }

        let data = &buffer.as_slice()[..aligned_len];
        let mut chunk_written = 0usize;
        while chunk_written < data.len() {
            let mut bytes_written: DWORD = 0;
            let ok = unsafe {
                WriteFile(
                    handle,
                    data.as_ptr().add(chunk_written) as *const c_void,
                    (data.len() - chunk_written) as DWORD,
                    &mut bytes_written,
                    ptr::null_mut(),
                )
            };

            if ok == 0 {
                let err = unsafe { GetLastError() };
                failure = Some(format!(
                    "WriteFile failed (Win32 error {}). Try closing Explorer windows for the SD card and ensure no partition is mounted.",
                    err
                ));
                break;
            }

            if bytes_written == 0 {
                failure = Some("WriteFile wrote 0 bytes unexpectedly.".to_string());
                break;
            }

            chunk_written += bytes_written as usize;
        }

        if failure.is_some() {
            break;
        }

        written_total += chunk_written as u64;
        logical_done = (logical_done + logical_len as u64).min(image_size);
        let eta = estimate_remaining_time(start_time.elapsed(), logical_done, image_size);
        on_progress(TransferProgress {
            done: logical_done,
            total: image_size,
            eta,
        });
    }

    let _ = reader_thread.join();
    unsafe { CloseHandle(handle) };
    for volume_handle in locked_volume_handles {
        unsafe { CloseHandle(volume_handle) };
    }

    if let Some(err) = failure {
        return Err(err);
    }

    on_progress(TransferProgress {
        done: image_size,
        total: image_size,
        eta: None,
    });

    Ok(())
}

fn parse_physical_drive_index(device_name: &str) -> Option<u32> {
    let digits_rev: String = device_name
        .chars()
        .rev()
        .take_while(|ch| ch.is_ascii_digit())
        .collect();

    if digits_rev.is_empty() {
        return None;
    }

    digits_rev.chars().rev().collect::<String>().parse::<u32>().ok()
}

fn lock_and_dismount_volumes_for_disk(disk_index: u32) -> Result<Vec<*mut c_void>, String> {
    let mut locked_handles = Vec::new();
    let drives_mask = unsafe { GetLogicalDrives() };

    for i in 0..26 {
        if (drives_mask & (1 << i)) == 0 {
            continue;
        }

        let letter = (b'A' + i as u8) as char;
        let volume_path = format!("\\\\.\\{}:", letter);
        let mut utf16: Vec<u16> = volume_path.encode_utf16().collect();
        utf16.push(0);

        let volume_handle = unsafe {
            CreateFileW(
                utf16.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                ptr::null_mut(),
                OPEN_EXISTING,
                0,
                ptr::null_mut(),
            ) as *mut c_void
        };

        if volume_handle == INVALID_HANDLE_VALUE {
            continue;
        }

        let mut extent_buffer = [0u8; 512];
        let mut bytes_returned: DWORD = 0;
        let ok = unsafe {
            DeviceIoControl(
                volume_handle,
                IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS,
                ptr::null_mut(),
                0,
                extent_buffer.as_mut_ptr() as *mut c_void,
                extent_buffer.len() as DWORD,
                &mut bytes_returned,
                ptr::null_mut(),
            )
        };

        if ok == 0 {
            unsafe { CloseHandle(volume_handle) };
            continue;
        }

        let extents = unsafe { &*(extent_buffer.as_ptr() as *const VOLUME_DISK_EXTENTS) };
        let extent_count = extents.NumberOfDiskExtents as usize;
        let first_extent_ptr = extents.Extents.as_ptr() as *const DISK_EXTENT;
        let mut belongs_to_target_disk = false;

        for extent_idx in 0..extent_count {
            let extent = unsafe { &*first_extent_ptr.add(extent_idx) };
            if extent.DiskNumber == disk_index {
                belongs_to_target_disk = true;
                break;
            }
        }

        if !belongs_to_target_disk {
            unsafe { CloseHandle(volume_handle) };
            continue;
        }

        let lock_ok = unsafe {
            DeviceIoControl(
                volume_handle,
                FSCTL_LOCK_VOLUME,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                0,
                &mut bytes_returned,
                ptr::null_mut(),
            )
        };

        if lock_ok == 0 {
            let err = unsafe { GetLastError() };
            unsafe { CloseHandle(volume_handle) };
            for handle in locked_handles {
                unsafe { CloseHandle(handle) };
            }
            return Err(format!(
                "Could not lock mounted volume {} (Win32 error {}). Close apps using the SD card and retry.",
                volume_path, err
            ));
        }

        let dismount_ok = unsafe {
            DeviceIoControl(
                volume_handle,
                FSCTL_DISMOUNT_VOLUME,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                0,
                &mut bytes_returned,
                ptr::null_mut(),
            )
        };

        if dismount_ok == 0 {
            let err = unsafe { GetLastError() };
            unsafe { CloseHandle(volume_handle) };
            for handle in locked_handles {
                unsafe { CloseHandle(handle) };
            }
            return Err(format!(
                "Could not dismount volume {} (Win32 error {}). Close Explorer windows and retry.",
                volume_path, err
            ));
        }

        locked_handles.push(volume_handle);
    }

    Ok(locked_handles)
}

fn open_device_for_read(device_name: &str) -> *mut c_void {
    let mut utf16: Vec<u16> = device_name.encode_utf16().collect();
    utf16.push(0);

    unsafe {
        CreateFileW(
            utf16.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            ptr::null_mut(),
            OPEN_EXISTING,
            FILE_FLAG_SEQUENTIAL_SCAN,
            ptr::null_mut(),
        ) as *mut c_void
    }
}

fn open_device_for_write(device_name: &str) -> *mut c_void {
    let mut utf16: Vec<u16> = device_name.encode_utf16().collect();
    utf16.push(0);

    unsafe {
        CreateFileW(
            utf16.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            ptr::null_mut(),
            OPEN_EXISTING,
            // Write-through bypasses the system cache so throughput reflects the card's real
            // sustained speed instead of bursting into RAM then stalling once the cache fills.
            // No buffering bypasses the system cache entirely (direct-to-media I/O), which is
            // what actually eliminates the cache-fill/stall bursts; write-through is kept as a
            // belt-and-braces guarantee that nothing is silently cached by lower layers.
            FILE_FLAG_SEQUENTIAL_SCAN | FILE_FLAG_WRITE_THROUGH | FILE_FLAG_NO_BUFFERING,
            ptr::null_mut(),
        ) as *mut c_void
    }
}

pub fn format_capacity(bytes: u64) -> String {
    if bytes == 0 {
        return "capacity unknown".to_string();
    }

    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = bytes as f64;
    let mut unit = 0usize;

    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }

    format!("{:7.2} {:>3}", size, UNITS[unit])
}

fn estimate_remaining_time(elapsed: Duration, done: u64, total: u64) -> Option<Duration> {
    if done == 0 || total == 0 || done >= total {
        return None;
    }

    let elapsed_nanos = elapsed.as_nanos();
    if elapsed_nanos == 0 {
        return None;
    }

    let remaining = total - done;
    let estimated_remaining_nanos = elapsed_nanos.saturating_mul(remaining as u128)/ done as u128;

    Some(Duration::from_nanos(estimated_remaining_nanos.min(u64::MAX as u128) as u64))
}

