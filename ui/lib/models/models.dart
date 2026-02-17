import 'package:equatable/equatable.dart';

/// Contact model
class Contact extends Equatable {
  final String address;
  final String nickname;
  final bool online;
  final DateTime? lastSeen;

  const Contact({
    required this.address,
    required this.nickname,
    this.online = false,
    this.lastSeen,
  });

  Contact copyWith({
    String? address,
    String? nickname,
    bool? online,
    DateTime? lastSeen,
  }) {
    return Contact(
      address: address ?? this.address,
      nickname: nickname ?? this.nickname,
      online: online ?? this.online,
      lastSeen: lastSeen ?? this.lastSeen,
    );
  }

  @override
  List<Object?> get props => [address, nickname, online, lastSeen];
}

/// Message model
class Message extends Equatable {
  final String id;
  final String sender;
  final String content;
  final DateTime timestamp;
  final MessageType type;
  final bool isOutgoing;
  final MessageStatus status;
  final FileMetadata? file;

  const Message({
    required this.id,
    required this.sender,
    required this.content,
    required this.timestamp,
    required this.type,
    required this.isOutgoing,
    this.status = MessageStatus.sent,
    this.file,
  });

  Message copyWith({
    String? id,
    String? sender,
    String? content,
    DateTime? timestamp,
    MessageType? type,
    bool? isOutgoing,
    MessageStatus? status,
    FileMetadata? file,
  }) {
    return Message(
      id: id ?? this.id,
      sender: sender ?? this.sender,
      content: content ?? this.content,
      timestamp: timestamp ?? this.timestamp,
      type: type ?? this.type,
      isOutgoing: isOutgoing ?? this.isOutgoing,
      status: status ?? this.status,
      file: file ?? this.file,
    );
  }

  @override
  List<Object?> get props => [id, sender, content, timestamp, type, isOutgoing, status];
}

enum MessageType { text, file, handshake, keepAlive, disconnect }

enum MessageStatus { sending, sent, delivered, read, failed }

/// File metadata
class FileMetadata extends Equatable {
  final String name;
  final int size;
  final String mimeType;

  const FileMetadata({
    required this.name,
    required this.size,
    required this.mimeType,
  });

  @override
  List<Object?> get props => [name, size, mimeType];
}

/// Identity model
class Identity extends Equatable {
  final String onionAddress;
  final String publicKey;
  final DateTime createdAt;

  const Identity({
    required this.onionAddress,
    required this.publicKey,
    required this.createdAt,
  });

  String get shortAddress {
    if (onionAddress.length > 16) {
      return '${onionAddress.substring(0, 8)}...${onionAddress.substring(onionAddress.length - 6)}';
    }
    return onionAddress;
  }

  @override
  List<Object?> get props => [onionAddress, publicKey, createdAt];
}

/// Tor status
enum TorConnectionStatus {
  disconnected,
  connecting,
  ready,
  error,
}

/// Chat session
class ChatSession extends Equatable {
  final Contact contact;
  final List<Message> messages;
  final bool isConnected;

  const ChatSession({
    required this.contact,
    required this.messages,
    this.isConnected = false,
  });

  ChatSession copyWith({
    Contact? contact,
    List<Message>? messages,
    bool? isConnected,
  }) {
    return ChatSession(
      contact: contact ?? this.contact,
      messages: messages ?? this.messages,
      isConnected: isConnected ?? this.isConnected,
    );
  }

  @override
  List<Object?> get props => [contact, messages, isConnected];
}
