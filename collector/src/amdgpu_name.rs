use std::{
    ffi::{CStr, c_char, c_int, c_void},
    fs,
    os::fd::AsRawFd,
    path::{Path, PathBuf},
};

use libloading::{Library, Symbol};

type AmdgpuDeviceHandle = *mut c_void;

type DeviceInitialize = unsafe extern "C" fn(
    fd: c_int,
    major_version: *mut u32,
    minor_version: *mut u32,
    device_handle: *mut AmdgpuDeviceHandle,
) -> c_int;

type DeviceDeinitialize = unsafe extern "C" fn(device_handle: AmdgpuDeviceHandle) -> c_int;

type GetMarketingName = unsafe extern "C" fn(device_handle: AmdgpuDeviceHandle) -> *const c_char;

/// Retrieves the marketing name of an AMD GPU using libdrm.
///
/// Returns None when libdrm or the GPU render node is unavailable.
pub fn gpu_name(pci_device: &Path) -> Option<String> {
    let pci_device = fs::canonicalize(pci_device).ok()?;
    let render_node = find_render_node(&pci_device)?;

    let file = fs::OpenOptions::new().read(true).write(true).open(render_node).ok()?;

    // SAFETY: All symbols are resolved from the loaded library.
    // The library remains loaded until the device handle is released.
    unsafe {
        let library = Library::new("libdrm_amdgpu.so.1").ok()?;

        let initialize: Symbol<DeviceInitialize> = library.get(b"amdgpu_device_initialize\0").ok()?;

        let deinitialize: Symbol<DeviceDeinitialize> = library.get(b"amdgpu_device_deinitialize\0").ok()?;

        let get_marketing_name: Symbol<GetMarketingName> = library.get(b"amdgpu_get_marketing_name\0").ok()?;

        let mut major = 0;
        let mut minor = 0;
        let mut handle: AmdgpuDeviceHandle = std::ptr::null_mut();

        if initialize(file.as_raw_fd(), &mut major, &mut minor, &mut handle) != 0 {
            return None;
        }

        if handle.is_null() {
            return None;
        }

        let name_ptr = get_marketing_name(handle);

        let name = if name_ptr.is_null() {
            None
        } else {
            let name = CStr::from_ptr(name_ptr).to_string_lossy().trim().to_owned();

            (!name.is_empty()).then_some(name)
        };

        // The returned name is owned by libdrm.
        // Do not free the name pointer.
        deinitialize(handle);

        name
    }
}

/// Finds the render node associated with a PCI device.
fn find_render_node(pci_device: &Path) -> Option<PathBuf> {
    for entry in fs::read_dir(pci_device.join("drm")).ok()?.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if let Some(index) = name.strip_prefix("renderD")
            && !index.is_empty()
            && index.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Some(Path::new("/dev/dri").join(name.as_ref()));
        }
    }

    None
}
