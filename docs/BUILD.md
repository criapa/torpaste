# TorChat-Paste Build Guide

## Prerequisites

### Common Requirements
- Flutter SDK 3.0 or higher
- Rust toolchain (latest stable)
- Git

### Platform-Specific Requirements

#### Android
- Android SDK (API 21+)
- Gradle 7.0+
- Android NDK (for libsodium)

#### iOS
- Xcode 14.0+ (macOS only)
- CocoaPods
- iOS 12.0+ deployment target

#### Windows
- Visual Studio Build Tools 2022
- Windows 10/11 SDK

#### Linux
- GCC/Clang
- CMake
- libssl-dev

#### macOS
- Xcode 14.0+
- Homebrew
- CocoaPods

## Build Steps

### 1. Clone Repository
```bash
git clone https://github.com/yourusername/torchat_paste.git
cd torchat_paste
```

### 2. Install Dependencies

#### Flutter
```bash
cd ui
flutter pub get
```

#### Rust
```bash
cd core
cargo build --release
```

### 3. Build for Platform

#### Android
```bash
cd ui
flutter build apk --release
# Output: build/app/outputs/flutter-apk/app-release.apk
```

#### iOS (macOS only)
```bash
cd ui
flutter build ios --release
```

#### Windows
```bash
cd ui
flutter build windows --release
```

#### Linux
```bash
cd ui
flutter build linux --release
```

#### macOS
```bash
cd ui
flutter build macos --release
```

## Docker Build (Linux)

```bash
# Build using Docker
docker build -t torchat_paste .
```

## Troubleshooting

### libsodium Build Errors
```bash
# Install libsodium development library
# Ubuntu/Debian
sudo apt-get install libsodium-dev

# macOS
brew install libsodium
```

### Tor Connection Issues
- Ensure network connectivity
- Check firewall settings for Tor ports (9050, 9051)
- Verify Tor is not already running

### Flutter Build Errors
```bash
# Clean and rebuild
flutter clean
flutter pub get
flutter build <platform>
```

## Configuration

### Pastebin API Key
Edit `ui/lib/services/pastebin_service.dart`:
```dart
void setApiKey(String key) {
  _apiKey = key; // Get free key from pastebin.com/api
}
```

### Tor Binary Path
Configure Tor binary location in `core/src/config.rs`.

## Output Locations

| Platform | Path |
|----------|------|
| Android APK | `ui/build/app/outputs/flutter-apk/` |
| iOS IPA | `ui/build/ios/iphoneos/Runner.ipa` |
| Windows EXE | `ui/build/windows/runner/Release/` |
| Linux AppImage | `ui/build/linux/x64/release/bundle/` |
| macOS App | `ui/build/macos/Build/Products/Release/` |
