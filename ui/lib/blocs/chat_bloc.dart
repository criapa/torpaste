import 'dart:async';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:equatable/equatable.dart';
import '../services/chat_service.dart';
import '../models/models.dart';

// Events
abstract class ChatEvent extends Equatable {
  const ChatEvent();

  @override
  List<Object?> get props => [];
}

class ChatLoadHistory extends ChatEvent {
  final String address;

  const ChatLoadHistory(this.address);

  @override
  List<Object?> get props => [address];
}

class ChatSendMessage extends ChatEvent {
  final String address;
  final String content;

  const ChatSendMessage({required this.address, required this.content});

  @override
  List<Object?> get props => [address, content];
}

class ChatSendFile extends ChatEvent {
  final String address;
  final String fileName;
  final int fileSize;
  final String mimeType;

  const ChatSendFile({
    required this.address,
    required this.fileName,
    required this.fileSize,
    required this.mimeType,
  });

  @override
  List<Object?> get props => [address, fileName, fileSize, mimeType];
}

class ChatMessageReceived extends ChatEvent {
  final Message message;

  const ChatMessageReceived(this.message);

  @override
  List<Object?> get props => [message];
}

class ChatConnect extends ChatEvent {
  final String address;

  const ChatConnect(this.address);

  @override
  List<Object?> get props => [address];
}

class ChatDisconnect extends ChatEvent {
  final String address;

  const ChatDisconnect(this.address);

  @override
  List<Object?> get props => [address];
}

class ChatConnectionChanged extends ChatEvent {
  final String address;
  final bool isConnected;

  const ChatConnectionChanged({required this.address, required this.isConnected});

  @override
  List<Object?> get props => [address, isConnected];
}

class ChatClearHistory extends ChatEvent {
  final String address;

  const ChatClearHistory(this.address);

  @override
  List<Object?> get props => [address];
}

// State
class ChatState extends Equatable {
  final String? currentContact;
  final List<Message> messages;
  final Map<String, bool> connections;
  final bool isLoading;
  final bool isSending;
  final String? error;

  const ChatState({
    this.currentContact,
    this.messages = const [],
    this.connections = const {},
    this.isLoading = false,
    this.isSending = false,
    this.error,
  });

  ChatState copyWith({
    String? currentContact,
    List<Message>? messages,
    Map<String, bool>? connections,
    bool? isLoading,
    bool? isSending,
    String? error,
  }) {
    return ChatState(
      currentContact: currentContact ?? this.currentContact,
      messages: messages ?? this.messages,
      connections: connections ?? this.connections,
      isLoading: isLoading ?? this.isLoading,
      isSending: isSending ?? this.isSending,
      error: error,
    );
  }

  @override
  List<Object?> get props => [currentContact, messages, connections, isLoading, isSending, error];
}

// BLoC
class ChatBloc extends Bloc<ChatEvent, ChatState> {
  final ChatService chatService;
  StreamSubscription? _messageSubscription;
  StreamSubscription? _connectionSubscription;

  ChatBloc({required this.chatService}) : super(const ChatState()) {
    on<ChatLoadHistory>(_onLoadHistory);
    on<ChatSendMessage>(_onSendMessage);
    on<ChatSendFile>(_onSendFile);
    on<ChatMessageReceived>(_onMessageReceived);
    on<ChatConnect>(_onConnect);
    on<ChatDisconnect>(_onDisconnect);
    on<ChatConnectionChanged>(_onConnectionChanged);
    on<ChatClearHistory>(_onClearHistory);

    // Listen to chat service streams
    _messageSubscription = chatService.messageStream.listen((message) {
      add(ChatMessageReceived(message));
    });

    _connectionSubscription = chatService.connectionStream.listen((entry) {
      add(ChatConnectionChanged(address: entry.key, isConnected: entry.value));
    });
  }

  Future<void> _onLoadHistory(ChatLoadHistory event, Emitter<ChatState> emit) async {
    emit(state.copyWith(isLoading: true, currentContact: event.address));

    try {
      final messages = chatService.getChatHistory(event.address);
      emit(state.copyWith(messages: messages, isLoading: false));
    } catch (e) {
      emit(state.copyWith(error: e.toString(), isLoading: false));
    }
  }

  Future<void> _onSendMessage(ChatSendMessage event, Emitter<ChatState> emit) async {
    emit(state.copyWith(isSending: true));

    try {
      final message = await chatService.sendMessage(
        toAddress: event.address,
        content: event.content,
      );

      final messages = List<Message>.from(state.messages)..add(message);
      emit(state.copyWith(messages: messages, isSending: false));
    } catch (e) {
      emit(state.copyWith(error: e.toString(), isSending: false));
    }
  }

  Future<void> _onSendFile(ChatSendFile event, Emitter<ChatState> emit) async {
    emit(state.copyWith(isSending: true));

    try {
      final message = await chatService.sendFile(
        toAddress: event.address,
        fileName: event.fileName,
        fileSize: event.fileSize,
        mimeType: event.mimeType,
      );

      final messages = List<Message>.from(state.messages)..add(message);
      emit(state.copyWith(messages: messages, isSending: false));
    } catch (e) {
      emit(state.copyWith(error: e.toString(), isSending: false));
    }
  }

  void _onMessageReceived(ChatMessageReceived event, Emitter<ChatState> emit) {
    if (event.message.sender == state.currentContact) {
      final messages = List<Message>.from(state.messages)..add(event.message);
      emit(state.copyWith(messages: messages));
    }
  }

  Future<void> _onConnect(ChatConnect event, Emitter<ChatState> emit) async {
    try {
      await chatService.connect(event.address);
    } catch (e) {
      emit(state.copyWith(error: e.toString()));
    }
  }

  Future<void> _onDisconnect(ChatDisconnect event, Emitter<ChatState> emit) async {
    await chatService.disconnect(event.address);
  }

  void _onConnectionChanged(ChatConnectionChanged event, Emitter<ChatState> emit) {
    final connections = Map<String, bool>.from(state.connections);
    connections[event.address] = event.isConnected;
    emit(state.copyWith(connections: connections));
  }

  void _onClearHistory(ChatClearHistory event, Emitter<ChatState> emit) {
    chatService.clearChatHistory(event.address);
    emit(state.copyWith(messages: []));
  }

  @override
  Future<void> close() {
    _messageSubscription?.cancel();
    _connectionSubscription?.cancel();
    chatService.dispose();
    return super.close();
  }
}
