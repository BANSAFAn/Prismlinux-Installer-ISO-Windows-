use futures_util::StreamExt;
use std::io::{Read, Write};
use std::os::windows::io::FromRawHandle;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

use windows_sys::Win32::Foundation::{
    CloseHandle, HANDLE, INVALID_HANDLE_VALUE, GENERIC_READ, GENERIC_WRITE,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL,
};
use windows_sys::Win32::System::Ioctl::{
    FSCTL_LOCK_VOLUME, FSCTL_DISMOUNT_VOLUME, FSCTL_UNLOCK_VOLUME,
};
use windows_sys::Win32::System::IO::DeviceIoControl;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct UsbDrive {
    pub number: u32,
    pub friendly_name: String,
    pub size: u64,
}

unsafe fn lock_volume(handle: HANDLE) -> bool {
    let mut returned: u32 = 0;
    DeviceIoControl(
        handle,
        FSCTL_LOCK_VOLUME,
        std::ptr::null(),
        0,
        std::ptr::null_mut(),
        0,
        &mut returned,
        std::ptr::null_mut(),
    ) != 0
}

unsafe fn dismount_volume(handle: HANDLE) -> bool {
    let mut returned: u32 = 0;
    DeviceIoControl(
        handle,
        FSCTL_DISMOUNT_VOLUME,
        std::ptr::null(),
        0,
        std::ptr::null_mut(),
        0,
        &mut returned,
        std::ptr::null_mut(),
    ) != 0
}

unsafe fn unlock_volume(handle: HANDLE) -> bool {
    let mut returned: u32 = 0;
    DeviceIoControl(
        handle,
        FSCTL_UNLOCK_VOLUME,
        std::ptr::null(),
        0,
        std::ptr::null_mut(),
        0,
        &mut returned,
        std::ptr::null_mut(),
    ) != 0
}

fn parse_usb_drives(json_str: &str) -> Vec<UsbDrive> {
    let s = json_str.trim();
    if s.is_empty() {
        return vec![];
    }
    if let Ok(drives) = serde_json::from_str::<Vec<UsbDrive>>(s) {
        drives
    } else if let Ok(drive) = serde_json::from_str::<UsbDrive>(s) {
        vec![drive]
    } else {
        vec![]
    }
}

#[tauri::command]
pub async fn get_usb_drives() -> Result<Vec<UsbDrive>, String> {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-Disk | Where-Object BusType -eq 'USB' | Select-Object Number, FriendlyName, Size | ConvertTo-Json -Compress",
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("PowerShell error listing drives: {}", err_msg));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    Ok(parse_usb_drives(&json_str))
}

#[tauri::command]
pub async fn download_iso(
    window: tauri::Window,
    url: String,
    save_path: String,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "PrismLinuxInstaller/1.0")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    
    if let Some(parent) = std::path::Path::new(&save_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut file = std::fs::File::create(&save_path).map_err(|e| e.to_string())?;
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    
    let start_time = std::time::Instant::now();
    let mut last_emit = std::time::Instant::now();

    #[derive(Clone, Serialize)]
    struct Progress {
        downloaded: u64,
        total: u64,
        speed: f64,
    }

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        let now = std::time::Instant::now();
        if now.duration_since(last_emit).as_millis() > 200 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 {
                downloaded as f64 / elapsed
            } else {
                0.0
            };

            let _ = window.emit(
                "download-progress",
                Progress {
                    downloaded,
                    total: total_size,
                    speed,
                },
            );
            last_emit = now;
        }
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let speed = if elapsed > 0.0 {
        downloaded as f64 / elapsed
    } else {
        0.0
    };
    let _ = window.emit(
        "download-progress",
        Progress {
            downloaded,
            total: total_size,
            speed,
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn flash_usb_drive(
    window: tauri::Window,
    drive_num: u32,
    iso_path: String,
) -> Result<(), String> {
    let usb_drives = get_usb_drives().await?;
    let found = usb_drives.iter().any(|d| d.number == drive_num);
    if !found {
        return Err("SAFETY VIOLATION: The selected drive is not a verified USB device!".into());
    }

    let iso_file = std::fs::File::open(&iso_path).map_err(|e| format!("Failed to open ISO file: {}", e))?;
    let total_size = iso_file.metadata().map_err(|e| e.to_string())?.len();

    let drive_path = format!("\\\\.\\PhysicalDrive{}", drive_num);
    let mut drive_path_w: Vec<u16> = drive_path.encode_utf16().collect();
    drive_path_w.push(0);

    unsafe {
        let handle = CreateFileW(
            drive_path_w.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            std::ptr::null_mut(),
        );

        if handle == INVALID_HANDLE_VALUE {
            return Err("Failed to open physical drive. Please ensure you are running as Administrator.".into());
        }

        if !lock_volume(handle) {
            CloseHandle(handle);
            return Err("Failed to lock volume. Please close any open files or folders on the USB drive.".into());
        }

        if !dismount_volume(handle) {
            unlock_volume(handle);
            CloseHandle(handle);
            return Err("Failed to dismount volume.".into());
        }

        let mut drive_file = std::fs::File::from_raw_handle(handle as *mut std::ffi::c_void);
        let mut reader = std::io::BufReader::new(iso_file);
        let mut buffer = vec![0u8; 1024 * 1024];
        let mut written: u64 = 0;
        
        let start_time = std::time::Instant::now();
        let mut last_emit = std::time::Instant::now();

        #[derive(Clone, Serialize)]
        struct Progress {
            written: u64,
            total: u64,
            speed: f64,
        }

        loop {
            let n = reader.read(&mut buffer).map_err(|e| format!("Read error: {}", e))?;
            if n == 0 {
                break;
            }
            drive_file.write_all(&buffer[..n]).map_err(|e| format!("Write error: {}", e))?;
            written += n as u64;

            let now = std::time::Instant::now();
            if now.duration_since(last_emit).as_millis() > 200 {
                let elapsed = start_time.elapsed().as_secs_f64();
                let speed = if elapsed > 0.0 {
                    written as f64 / elapsed
                } else {
                    0.0
                };

                let _ = window.emit(
                    "flash-progress",
                    Progress {
                        written,
                        total: total_size,
                        speed,
                    },
                );
                last_emit = now;
            }
        }

        drive_file.flush().map_err(|e| e.to_string())?;
        unlock_volume(handle);
    }

    Ok(())
}

#[tauri::command]
pub async fn setup_direct_boot(iso_path: String) -> Result<(), String> {
    let mount_output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", "mountvol S: /S"])
        .output()
        .map_err(|e| e.to_string())?;
    
    if !mount_output.status.success() {
        return Err("Failed to mount EFI partition. Ensure the app has Administrator privileges.".into());
    }

    struct MountGuard;
    impl Drop for MountGuard {
        fn drop(&mut self) {
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", "mountvol S: /D"])
                .output();
        }
    }
    let _guard = MountGuard;

    let iso_path_abs = std::path::Path::new(&iso_path)
        .canonicalize()
        .map_err(|e| format!("Invalid ISO file path: {}", e))?
        .to_string_lossy()
        .to_string()
        .replace("\\\\?\\", "");

    let mount_iso_cmd = format!(
        "$mount = Mount-DiskImage -ImagePath '{}' -PassThru; ($mount | Get-Volume).DriveLetter",
        iso_path_abs
    );
    let iso_mount_output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &mount_iso_cmd])
        .output()
        .map_err(|e| e.to_string())?;

    if !iso_mount_output.status.success() {
        return Err("Failed to mount ISO to retrieve kernel/initramfs paths.".into());
    }

    let drive_letter = String::from_utf8_lossy(&iso_mount_output.stdout).trim().to_string();
    if drive_letter.is_empty() {
        return Err("Failed to assign a drive letter to mounted ISO.".into());
    }

    let iso_drive = format!("{}:\\", drive_letter);

    let mut kernel_src = format!("{}arch\\boot\\x86_64\\vmlinuz-linux", iso_drive);
    let mut initramfs_src = format!("{}arch\\boot\\x86_64\\initramfs-linux.img", iso_drive);

    if !std::path::Path::new(&kernel_src).exists() {
        kernel_src = format!("{}boot\\vmlinuz-x86_64", iso_drive);
        initramfs_src = format!("{}boot\\initramfs-x86_64.img", iso_drive);
        if !std::path::Path::new(&kernel_src).exists() {
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", &format!("Dismount-DiskImage -ImagePath '{}'", iso_path_abs)])
                .output();
            return Err("Failed to find kernel and initramfs files inside the ISO. The ISO might be corrupted or unsupported.".into());
        }
    }

    std::fs::create_dir_all("C:\\PrismLinux").map_err(|e| e.to_string())?;
    std::fs::copy(&kernel_src, "C:\\PrismLinux\\vmlinuz").map_err(|e| e.to_string())?;
    std::fs::copy(&initramfs_src, "C:\\PrismLinux\\initramfs.img").map_err(|e| e.to_string())?;
    std::fs::copy(&iso_path_abs, "C:\\PrismLinux\\prism.iso").map_err(|e| e.to_string())?;

    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &format!("Dismount-DiskImage -ImagePath '{}'", iso_path_abs)])
        .output();

    std::fs::create_dir_all("S:\\EFI\\PrismLinux").map_err(|e| e.to_string())?;

    let grub_url = "http://archive.ubuntu.com/ubuntu/dists/jammy/main/uefi/grub2-amd64/current/grubnetx64.efi.signed";
    let client = reqwest::Client::new();
    let grub_bytes = client.get(grub_url)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Ubuntu bootloader server: {}", e))?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;

    std::fs::write("S:\\EFI\\PrismLinux\\grubx64.efi", &grub_bytes).map_err(|e| e.to_string())?;

    let uuid_output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", "(Get-Volume -DriveLetter C).Id"])
        .output()
        .map_err(|e| e.to_string())?;

    let vol_id = String::from_utf8_lossy(&uuid_output.stdout).trim().to_string();
    let uuid = if let Some(start) = vol_id.find('{') {
        if let Some(end) = vol_id.find('}') {
            &vol_id[start + 1..end]
        } else {
            ""
        }
    } else {
        ""
    };

    if uuid.is_empty() {
        return Err("Failed to read NTFS volume ID for C: drive.".into());
    }

    let grub_cfg = format!(
        r#"set default=0
set timeout=5

menuentry "Prism Linux Live Installer" --class arch {{
    search --no-floppy --set=root --file /PrismLinux/prism.iso
    probe -u ($root) --set=uuid
    linux /PrismLinux/vmlinuz img_dev=/dev/disk/by-uuid/$uuid img_loop=/PrismLinux/prism.iso earlymodules=loop
    initrd /PrismLinux/initramfs.img
}}
"#
    );

    std::fs::write("S:\\EFI\\PrismLinux\\grub.cfg", grub_cfg).map_err(|e| e.to_string())?;

    let bcd_create = std::process::Command::new("cmd")
        .args(["/C", "bcdedit /create /d \"Prism Linux Installer\" /application OSLOADER"])
        .output()
        .map_err(|e| e.to_string())?;

    if !bcd_create.status.success() {
        return Err("Failed to create BCD boot entry.".into());
    }

    let bcd_str = String::from_utf8_lossy(&bcd_create.stdout).to_string();
    let bcd_guid = if let Some(start) = bcd_str.find('{') {
        if let Some(end) = bcd_str.find('}') {
            Some(&bcd_str[start..=end])
        } else {
            None
        }
    } else {
        None
    };

    let guid = bcd_guid.ok_or_else(|| "Failed to parse boot GUID from BCDEDIT.".to_string())?;

    let bcd_commands = [
        format!("bcdedit /set {} device partition=S:", guid),
        format!("bcdedit /set {} path \\EFI\\PrismLinux\\grubx64.efi", guid),
        format!("bcdedit /set {} locale en-US", guid),
        format!("bcdedit /displayorder {} /addlast", guid),
    ];

    for cmd in bcd_commands {
        let out = std::process::Command::new("cmd")
            .args(["/C", &cmd])
            .output()
            .map_err(|e| e.to_string())?;
        
        if !out.status.success() {
            let err_msg = String::from_utf8_lossy(&out.stderr).to_string();
            return Err(format!("BCDEDIT setup failed on command '{}': {}", cmd, err_msg));
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn select_local_iso() -> Result<String, String> {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.OpenFileDialog; $f.Filter = 'ISO files (*.iso)|*.iso'; $f.ShowDialog() | Out-Null; $f.FileName",
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err("File selection cancelled or failed.".into());
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        return Err("No file selected.".into());
    }
    Ok(path)
}

#[tauri::command]
pub async fn reboot_system() -> Result<(), String> {
    std::process::Command::new("shutdown")
        .args(["/r", "/t", "0"])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn exit_app() {
    std::process::exit(0);
}

