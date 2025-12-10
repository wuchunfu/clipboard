---
sidebar_position: 2
---

# Installation

Clipboard Manager supports major desktop operating systems. Please choose the appropriate installation method for your system.

## Download Installer

Please visit our [GitHub Releases](https://github.com/tipsxBase/clipboard/releases) page to download the latest installer.

### macOS Users

- **Download**: Select the `.dmg` file (e.g., `Clipboard-Manager_x.x.x_universal.dmg`).
- **Install**: Double-click the `.dmg` file and drag the app icon into the `Applications` folder.
- **Permissions**: Upon first launch, the system may prompt for accessibility permissions (to listen for clipboard shortcuts). Please follow the instructions to authorize in "System Settings -> Privacy & Security -> Accessibility".

### Windows Users

- **Download**: Select the `.msi` installer or `.exe` file.
- **Install**: Double-click the installer and follow the wizard to complete the installation.

### Linux Users

- **Download**: We provide `.deb` (Debian/Ubuntu) and `.AppImage` formats.
- **Install**:
  - `.deb`: Run `sudo dpkg -i clipboard_x.x.x_amd64.deb`
  - `.AppImage`: Grant execution permission `chmod +x clipboard_x.x.x.AppImage` and run directly.

## Common Issues

### macOS: "App is damaged and can't be opened"

If you encounter a message stating that the application is damaged and cannot be opened, or that it should be moved to the Trash, this is a common macOS security measure for apps that are not notarized by Apple.

To resolve this:

1. Open the **Terminal** app.
2. Run the following command (ensure the path matches your installation location):
   ```bash
   sudo xattr -cr /Applications/clipboard.app
   ```
3. Enter your system password if prompted.
4. Try opening the app again.

Alternatively, you can try right-clicking the app icon and selecting **Open**, then clicking **Open** in the dialog box.

### Windows: Microsoft Defender SmartScreen Warning

On Windows, you might see a blue window saying **"Windows protected your PC"**. This appears because the application does not yet have a high reputation certificate.

To install:

1. Click the **More info** link in the popup.
2. Click the **Run anyway** button at the bottom.

## Automatic Updates

The application has built-in automatic update functionality. When a new version is released, the app will automatically detect it in the background and prompt you to update. Simply click confirm, and the app will automatically download and install the latest version.
