# PrismLinux Installer

Official desktop installer and flasher for the PrismLinux operating system. The application enables users to easily write official ISO images to USB flash drives or configure live installer boot staging directly from a local hard drive on Windows.

## Technology Stack

- **Core (Backend)**: Rust (Tauri v2) for safe, low-level interaction with disks, partitions, and system boot managers.
- **Interface (Frontend)**: Vue 3, TypeScript, Vite.
- **Styling**: Pure CSS conforming to the flat design language of `prismlinux.org` (Zinc dark theme, no gradients, no external icon libraries, and no emojis).

## Features

### 1. Write to USB Flash Drive (USB Flashing)
- Automatic detection of USB drives on the system (via PowerShell on Windows and `lsblk` on Linux).
- Low-level raw device locking and unmounting before block-by-block writing.
- Prevents accidental overwriting of primary OS disks using system partition validation checks.
- Real-time status reporting: speed, total bytes written, and remaining time.

### 2. Direct Device Installation (Direct HDD Boot) — *Windows only*
- Staging and booting the installation media without needing a USB drive.
- Creates local `C:\PrismLinux` directory and stages the Linux kernel and ISO image.
- Mounts the hidden EFI system partition.
- Places a signed, Microsoft-compliant standalone GRUB bootloader into the EFI structure.
- Registers "Prism Linux Live Installer" in the Windows Boot Manager boot menu via `bcdedit`.
- **BitLocker Guard**: Displays a mandatory warning checklist to suspend drive encryption before rebooting.

### 3. Smart Localization & Language Switcher
- Supporting 8 languages: Ukrainian, English, German, Spanish, Arabic (with native Right-to-Left RTL interface formatting), Chinese, Japanese, and Hindi.
- Automatically reads the host OS language/locale settings on startup to load the localized translations.
- Premium interactive language switcher featuring smooth chevron rotation, hover borders, active highlight lines, and clean monospaced language badges.
- Automatically hides the "Direct HDD Install" card when launched on Linux, centering the USB Flasher module.

### 4. Exit Safeguards
- Native window close calls are intercepted on the Rust backend, prompting a clean exit modal featuring a custom vector geometric crying cat drawing.

## Development & Building

### Requirements
Ensure you have **Node.js** (v20+) and **Rust (cargo/rustc)** installed on your machine.

### Run Development Webview
```bash
npm install
npm run dev
```

### Build Installers (CLI build tool)
Compile the production installer packages on your current host platform:
```bash
npm run build-cli
```
Optional debug builds:
```bash
node build.js --debug
```
Bundled binaries (`.msi`/`.exe` on Windows or `.deb`/`.AppImage` on Linux) will be automatically moved to the root-level `dist-installers/` directory.

## CI/CD Architecture (GitHub Actions)

1. **Pull Request Creator (`pr-creator.yml`)**:
   - Pushes to any development or feature branch will automatically trigger the bot to create or update a Pull Request to `main`.
   - The Pull Request body automatically generates a change report listing all commits included in the branch.

2. **Release Builder (`ci.yml`)**:
   - Release binaries are not compiled on draft branches.
   - When the Pull Request from the bot is merged into `main`, the build workflow is triggered.
   - It compiles the installers on Windows and Linux runners, attaches the `.exe`, `.msi`, `.deb`, and `.AppImage` files to a new GitHub Release draft, and uploads the artifacts for direct download.
