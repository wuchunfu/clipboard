---
sidebar_position: 1
---

# Introduction

**Clipboard Manager** is a lightweight, secure, and modern clipboard management tool built with **Rust (Tauri v2)** and **Vue 3**.

It is designed to help you efficiently manage your copy history, supporting both text and images, while providing powerful search capabilities and privacy protection.

## ✨ Key Features

### 📋 Comprehensive History

Automatically records everything you copy, whether it's code snippets, important text, or screenshots. Never worry about losing what you just copied again.

### 🔍 Instant Smart Search

Built-in high-performance full-text search engine. Locate the history records you need in milliseconds just by typing keywords.

### 🖼️ Image Preview & Management

Supports not only text but also image formats perfectly. You can preview copied images directly in the list and copy them again with one click.

### 🔒 Privacy First

We understand the sensitivity of clipboard data:

- **Local Storage**: All data is stored only locally on your device (`~/.clipboard-manager/`) and is never uploaded to the cloud.
- **Encrypted Storage**: The database uses AES-GCM advanced encryption standard, ensuring data cannot be read even if stolen.
- **Sensitive Filtering**: Automatically identifies sensitive information like passwords and supports setting specific applications (e.g., 1Password) to be ignored.

### ⚡ Ultimate Performance

- **Rust Powered**: The backend is written in Rust, with extremely low memory usage and fast response speed.
- **Lazy Loading Optimization**: Specially optimized for large amounts of history records and large text, ensuring the interface is always smooth.

### ⌨️ Keyboard Friendly

- **Vim Style Navigation**: Supports `j` / `k` for moving up and down, `Ctrl+n` / `Ctrl+p` for quick selection.
- **Global Shortcuts**: Summon the panel from any application with a simple shortcut (default `Cmd+Shift+V`).

### 🚀 Advanced Productivity

- **Paste Stack**: Copy multiple items and paste them sequentially in a specific order. Perfect for form filling.
- **OCR (macOS)**: Extract text directly from images in your clipboard history.

## 🌍 Cross-Platform & Internationalization

- Supports macOS, Windows, and Linux.
- Built-in English and Simplified Chinese interfaces, automatically following the system language.
