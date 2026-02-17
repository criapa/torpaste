# TorChat-Paste Android Platform Specifics

## Overview
This document describes Android-specific implementation details for TorChat-Paste.

## Requirements
- Android 5.0 (API 21) or higher
- Internet permission
- Foreground service permission for background Tor

## Key Features

### 1. Foreground Service for Tor
Android requires apps to use a foreground service to run in the background:
```kotlin
// TorForegroundService.kt
class TorForegroundService : Service() {
    override fun onCreate() {
        super.onCreate()
        startForeground(NOTIFICATION_ID, createNotification())
    }
}
```

### 2. Secure Window
Prevent screenshots on Android:
```kotlin
window.setFlags(
    WindowManager.LayoutParams.FLAG_SECURE,
    WindowManager.LayoutParams.FLAG_SECURE
)
```

### 3. Key Storage
Use Android Keystore for secure key storage:
```kotlin
val keyStore = KeyStore.getInstance("AndroidKeyStore")
```

### 4. Network Security Config
Configure network security for Tor in `res/xml/network_security_config.xml`:
```xml
<network-security-config>
    <domain-config cleartextTrafficPermitted="false">
        <domain includeSubdomains="true">onion</domain>
    </domain-config>
</network-security-config>
```

## Permissions Required
```xml
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
```

## Building
```bash
cd ui
flutter build apk --release
```
