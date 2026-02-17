# TorChat-Paste Desktop Platform Specifics

## Overview
This document describes desktop-specific implementation details for Windows, Linux, and macOS.

## Common Features

### 1. System Tray
Desktop apps should minimize to system tray to maintain Tor connection:
```dart
import 'tray_manager';

await TrayManager.instance.setIcon('assets/icon.png');
await TrayManager.instance.setToolTip('TorChat-Paste');
```

### 2. Auto-start
Register the app to start on system boot.

### 3. Global Shortcuts
Add global shortcuts for quick access:
- Show/hide window
- Quick compose

## Windows Specific
- Use Windows Service for background Tor process
- MSI installer with code signing

## Linux Specific
- AppImage, Flatpak, or Snap packaging
- Systemd service for autostart

## macOS Specific
- .app bundle
- Menu bar icon
- LaunchAgent for autostart
