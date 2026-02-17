# TorChat-Paste Architecture

## System Design

### Overview
TorChat-Paste follows a layered architecture separating concerns between the user interface, business logic, and system-level operations.

### Layers

#### 1. UI Layer (Flutter)
- **Screens**: Splash, Home, Chat, Settings, Identity, AddContact
- **Widgets**: Reusable UI components
- **BLoC**: State management using flutter_bloc

#### 2. Bridge Layer
- flutter_rust_bridge for type-safe Dart-Rust communication
- Handles serialization/deserialization
- Manages async communication

#### 3. Core Layer (Rust)
- **TorManager**: Tor daemon integration, hidden service management
- **Crypto**: libsodium-based encryption operations
- **PastebinClient**: HTTP client for pastebin API
- **Protocol**: P2P messaging protocol implementation
- **Storage**: Secure local storage with key encryption

### Data Flow

```
User Action → UI Event → BLoC → Bridge → Rust Core
                    ↓                        ↓
              UI Update ← State ← Bridge ← Response
```

## Protocol Specification

### Message Format
```json
{
  "id": "uuid",
  "type": "text|file|handshake",
  "sender": "onion_address",
  "content": "encrypted_content",
  "timestamp": 1234567890,
  "sequence": 1
}
```

### Handshake Protocol
1. Client connects to server onion via SOCKS5
2. Exchange public keys (X25519)
3. Derive session keys (ECDH)
4. Begin encrypted communication

### Encryption
- **Key Exchange**: X25519 (Curve25519)
- **Message Encryption**: XChaCha20-Poly1305
- **Key Derivation**: Argon2id for password-based keys

## Storage Schema

### Identity (Encrypted)
```json
{
  "onion_address": "...",
  "private_key": "base64_encoded",
  "public_key": "base64_encoded",
  "created_at": "timestamp"
}
```

### Contacts (Plaintext)
```json
[
  {
    "address": "onion_address",
    "nickname": "display_name",
    "added_at": "timestamp"
  }
]
```

## Security Model

### Threat Model
- **Network Observer**: Protected by Tor
- **Server**: No server stores messages
- **Local Access**: Identity encrypted at rest

### Mitigations
- Perfect forward secrecy via ephemeral session keys
- Memory zeroization on exit
- No metadata persistence
- Paste expiration reduces exposure window
