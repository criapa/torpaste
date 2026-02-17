# TorChat-Paste iOS Platform Specifics

## Overview
This document describes iOS-specific implementation details for TorChat-Paste.

## Requirements
- iOS 12.0 or higher
- Xcode 14.0 or higher

## Key Features

### 1. Background Execution
iOS has strict background execution limits. The Tor connection should only run in foreground by default.

### 2. Keychain Storage
Use iOS Keychain for secure key storage:
```swift
let query: [String: Any] = [
    kSecClass: kSecClassGenericPassword,
    kSecAttrService: "com.torchatpaste.identity",
    kSecAttrAccount: "onion_key",
    kSecValueData: keyData
]
SecItemAdd(query as CFDictionary, nil)
```

### 3. Clipboard Security
Clear clipboard after timeout:
```swift
UIPasteboard.general.items = []
```

### 4. Screenshot Prevention
Set `isSecureTextEntry` or use a secure text field.

## Info.plist Additions
```xml
<key>UIBackgroundModes</key>
<array>
    <string>fetch</string>
    <string>processing</string>
</array>
<key>NSAppTransportSecurity</key>
<dict>
    <key>NSAllowsArbitraryLoads</key>
    <true/>
</dict>
```

## Building
```bash
cd ui
flutter build ios --release
```

## Note
For App Store distribution, you may need to explain the use of Tor networking in your app review.
