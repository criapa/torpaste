import 'dart:async';
import 'dart:convert';
import 'dart:math';
import 'package:uuid/uuid.dart';
import '../models/models.dart';

/// Service for managing P2P chat connections
class ChatService {
  final _messageController = StreamController<Message>.broadcast();
  final _connectionController = StreamController<MapEntry<String, bool>>.broadcast();

  final Map<String, List<Message>> _chatHistory = {};
  final Map<String, bool> _connections = {};

  Stream<Message> get messageStream => _messageController.stream;
  Stream<MapEntry<String, bool>> get connectionStream => _connectionController.stream;

  final _uuid = const Uuid();

  /// Send a message to a contact
  Future<Message> sendMessage({
    required String toAddress,
    required String content,
    String? fromAddress,
  }) async {
    final message = Message(
      id: _uuid.v4(),
      sender: fromAddress ?? 'me',
      content: content,
      timestamp: DateTime.now(),
      type: MessageType.text,
      isOutgoing: true,
      status: MessageStatus.sending,
    );

    // Store in local history
    _chatHistory.putIfAbsent(toAddress, () => []);
    _chatHistory[toAddress]!.add(message);

    // Simulate network delay and send
    await Future.delayed(const Duration(milliseconds: 300));

    // Simulate delivery (in production, this would be actual P2P)
    final delivered = message.copyWith(status: MessageStatus.delivered);
    _updateMessage(toAddress, delivered);

    // Also simulate receiving a response occasionally
    if (Random().nextBool() && Random().nextBool()) {
      _simulateResponse(toAddress);
    }

    return delivered;
  }

  /// Get chat history for a contact
  List<Message> getChatHistory(String address) {
    return _chatHistory[address] ?? [];
  }

  /// Get all chat histories
  Map<String, List<Message>> get allChatHistories => _chatHistory;

  /// Clear chat history for a contact
  void clearChatHistory(String address) {
    _chatHistory.remove(address);
  }

  /// Clear all chat history (ephemeral storage)
  void clearAllHistory() {
    _chatHistory.clear();
  }

  /// Update a message in history
  void _updateMessage(String address, Message updated) {
    final messages = _chatHistory[address];
    if (messages != null) {
      final index = messages.indexWhere((m) => m.id == updated.id);
      if (index != -1) {
        messages[index] = updated;
        _messageController.add(updated);
      }
    }
  }

  /// Simulate receiving a response
  void _simulateResponse(String fromAddress) async {
    await Future.delayed(const Duration(seconds: 2));

    final responses = [
      'Received your message!',
      'Interesting...',
      'OK',
      'Let me think about that.',
      '👍',
      'Thanks for letting me know.',
    ];

    final response = Message(
      id: _uuid.v4(),
      sender: fromAddress,
      content: responses[Random().nextInt(responses.length)],
      timestamp: DateTime.now(),
      type: MessageType.text,
      isOutgoing: false,
      status: MessageStatus.delivered,
    );

    _chatHistory.putIfAbsent(fromAddress, () => []);
    _chatHistory[fromAddress]!.add(response);
    _messageController.add(response);
  }

  /// Connect to a contact
  Future<bool> connect(String address) async {
    // Simulate connection attempt
    await Future.delayed(const Duration(milliseconds: 500));

    // Simulate success/failure
    final success = Random().nextBool();
    _connections[address] = success;
    _connectionController.add(MapEntry(address, success));

    return success;
  }

  /// Disconnect from a contact
  Future<void> disconnect(String address) async {
    _connections[address] = false;
    _connectionController.add(MapEntry(address, false));
  }

  /// Check if connected to a contact
  bool isConnected(String address) {
    return _connections[address] ?? false;
  }

  /// Send file (simulated)
  Future<Message> sendFile({
    required String toAddress,
    required String fileName,
    required int fileSize,
    required String mimeType,
    String? fromAddress,
  }) async {
    final message = Message(
      id: _uuid.v4(),
      sender: fromAddress ?? 'me',
      content: 'File transfer: $fileName',
      timestamp: DateTime.now(),
      type: MessageType.file,
      isOutgoing: true,
      status: MessageStatus.sending,
      file: FileMetadata(
        name: fileName,
        size: fileSize,
        mimeType: mimeType,
      ),
    );

    _chatHistory.putIfAbsent(toAddress, () => []);
    _chatHistory[toAddress]!.add(message);

    // Simulate file transfer
    await Future.delayed(const Duration(seconds: 2));

    final delivered = message.copyWith(status: MessageStatus.delivered);
    _updateMessage(toAddress, delivered);

    return delivered;
  }

  /// Dispose resources
  void dispose() {
    _messageController.close();
    _connectionController.close();
  }
}
