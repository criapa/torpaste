# TorChat-Paste

Anonymous P2P Messenger with Tor integration and Pastebin-based address exchange.

## Overview

TorChat-Paste is a privacy-focused instant messaging application that enables secure, anonymous peer-to-peer communication using the Tor network. It features:

- **End-to-End Encryption**: All messages are encrypted using libsodium (XChaCha20-Poly1305)
- **Tor Hidden Services**: Uses Tor v3 hidden services for identity
- **Pastebin Address Exchange**: Share your address via anonymous pastebin pastes
- **Ephemeral Storage**: No messages or metadata are persisted to disk
- **Multi-Platform**: Available on Android, iOS, Windows, Linux, and macOS

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Flutter UI Layer                     │
│  (Screens, Widgets, BLoC State Management)              │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────┐
│              Flutter Rust Bridge                        │
│  (Type-safe communication between Dart and Rust)         │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────┐
│                    Rust Core                             │
│  ├── Tor Manager      - Hidden service management        │
│  ├── Crypto           - Encryption/decryption            │
│  ├── Pastebin Client - Address sharing                  │
│  ├── Protocol        - P2P messaging protocol           │
│  └── Storage         - Secure key storage               │
└─────────────────────────────────────────────────────────┘
```

## Features

### Identity Management
- Generate Tor v3 hidden service address
- Persistent local identity with secure storage
- Export/import identity with password protection

### Address Exchange
- Create anonymous pastebin pastes with expiration
- Import contacts from paste URLs
- Direct onion address input support

### Messaging
- Real-time P2P messaging via Tor
- File transfer support
- Message encryption with perfect forward secrecy

### Privacy
- Zero message persistence
- No metadata logging
- Memory zeroization on exit

## Building

### Prerequisites
- Flutter SDK 3.0+
- Rust toolchain
- For Android: Android SDK
- For iOS: Xcode (macOS only)

### Build Steps

```bash
# Clone the repository
git clone https://github.com/yourusername/torchat_paste.git
cd torchat_paste

# Build Rust core
cd core
cargo build --release
cd ..

# Build Flutter app
cd ui
flutter pub get
flutter build apk --release   # Android
flutter build ios --release   # iOS
flutter build windows --release  # Windows
flutter build linux --release    # Linux
flutter build macos --release    # macOS
```

## Usage

### First Launch
1. App initializes Tor connection
2. Generates new onion address
3. Displays your identity

### Adding a Contact
1. Tap the + button
2. Choose paste link or direct address
3. Enter the contact's paste URL or onion address
4. Optionally set a nickname

### Sharing Your Address
1. Tap the share button (bottom right)
2. App creates a pastebin paste with your address
3. Share the generated URL with your contact

### Sending Messages
1. Select a contact from the list
2. Type and send messages
3. Messages are encrypted end-to-end

## Security Considerations

- Always share paste links via secure channels
- Pastes expire after 10 minutes by default
- Keep your identity backup password safe
- No password can recover lost identity keys

## License

MIT License - See LICENSE file for details.

## Disclaimer

This software is for educational and privacy purposes. Users are responsible for complying with local laws regarding encryption and anonymity tools.
