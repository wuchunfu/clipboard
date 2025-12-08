---
sidebar_position: 3
---

# User Guide

## Basic Operations

### Summon Interface

By default, you can use the following global shortcut to summon the clipboard history panel:

- **macOS**: `Cmd + Shift + V`
- **Windows / Linux**: `Ctrl + Shift + V`

> **Tip**: You can modify this shortcut in the application settings.

### List Navigation

To improve efficiency, we support multiple navigation methods so your hands don't have to leave the keyboard:

- **Move Up/Down**:
  - Arrow keys `↑` / `↓`
  - Vim style `j` (down) / `k` (up)
  - Emacs style `Ctrl+n` (down) / `Ctrl+p` (up)
- **Confirm/Paste**: Press `Enter` to paste the selected item into the current active window and automatically close the panel.
- **Preview Details**: Press `Space` to view detailed information of the currently selected item (useful for long text or large images).
- **Close Panel**: Press `Esc`.

## Search Function

There is a search box at the top of the panel. After opening the panel, type keywords directly to filter history records.

- Supports fuzzy search.
- Search scope includes text content and source application name.

## Data Management

### Storage Location

To ensure your data privacy, all data is stored locally on your device:

- **macOS**: `~/.clipboard-manager/`
- **Windows**: `C:\Users\<Username>\.clipboard-manager\`
- **Linux**: `~/.clipboard-manager/`

### Database Security

- The database file is named `history.db`.
- The database uses **AES-GCM** algorithm for encryption, and the key is stored in a local secure location. This means that even if someone obtains your database file, they cannot read its content without the key.

### Privacy Settings

In settings, you can:

- **Pause Recording**: Temporarily stop listening for clipboard changes.
- **Ignore Applications**: Add application names (e.g., `1Password`, `KeyChain`) that should not be recorded to protect sensitive information.
- **Clear History**: One-click delete all locally stored history records.
