import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:intl/intl.dart';
import '../blocs/chat_bloc.dart';
import '../blocs/tor_bloc.dart';
import '../models/models.dart';
import '../utils/theme.dart';
import '../widgets/message_bubble.dart';

class ChatScreen extends StatefulWidget {
  const ChatScreen({super.key});

  @override
  State<ChatScreen> createState() => _ChatScreenState();
}

class _ChatScreenState extends State<ChatScreen> {
  final _messageController = TextEditingController();
  final _scrollController = ScrollController();
  late Contact _contact;

  @override
  void initState() {
    super.initState();
    // Get contact from arguments
    final args = ModalRoute.of(context)?.settings.arguments;
    if (args != null && args is Contact) {
      _contact = args;
      // Load chat history
      context.read<ChatBloc>().add(ChatLoadHistory(_contact.address));
      // Try to connect
      context.read<ChatBloc>().add(ChatConnect(_contact.address));
    }
  }

  @override
  void dispose() {
    _messageController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              _contact.nickname,
              style: const TextStyle(fontSize: 16),
            ),
            BlocBuilder<ChatBloc, ChatState>(
              builder: (context, state) {
                final isConnected = state.connections[_contact.address] ?? false;
                return Text(
                  isConnected ? 'Connected' : 'Connecting...',
                  style: TextStyle(
                    fontSize: 12,
                    color: isConnected ? AppTheme.primaryColor : AppTheme.textSecondary,
                  ),
                );
              },
            ),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.more_vert),
            onPressed: _showOptions,
          ),
        ],
      ),
      body: Column(
        children: [
          // Messages list
          Expanded(
            child: BlocBuilder<ChatBloc, ChatState>(
              builder: (context, state) {
                if (state.messages.isEmpty) {
                  return _buildEmptyChat();
                }

                return ListView.builder(
                  controller: _scrollController,
                  padding: const EdgeInsets.all(16),
                  itemCount: state.messages.length,
                  itemBuilder: (context, index) {
                    final message = state.messages[index];
                    final showDate = index == 0 ||
                        !_isSameDay(
                          state.messages[index - 1].timestamp,
                          message.timestamp,
                        );

                    return Column(
                      children: [
                        if (showDate) _buildDateDivider(message.timestamp),
                        MessageBubble(
                          message: message,
                          onLongPress: () => _copyMessage(message),
                        ),
                      ],
                    );
                  },
                );
              },
            ),
          ),

          // Input area
          _buildInputArea(),
        ],
      ),
    );
  }

  Widget _buildEmptyChat() {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.chat_bubble_outline,
              size: 64,
              color: AppTheme.textSecondary.withOpacity(0.5),
            ),
            const SizedBox(height: 16),
            Text(
              'Start a conversation with ${_contact.nickname}',
              textAlign: TextAlign.center,
              style: const TextStyle(
                color: AppTheme.textSecondary,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              _contact.address,
              style: const TextStyle(
                fontSize: 12,
                fontFamily: 'RobotoMono',
                color: AppTheme.textSecondary,
              ),
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildDateDivider(DateTime date) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 16),
      child: Row(
        children: [
          const Expanded(child: Divider(color: AppTheme.dividerColor)),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Text(
              DateFormat('MMM d, y').format(date),
              style: const TextStyle(
                fontSize: 12,
                color: AppTheme.textSecondary,
              ),
            ),
          ),
          const Expanded(child: Divider(color: AppTheme.dividerColor)),
        ],
      ),
    );
  }

  Widget _buildInputArea() {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: const BoxDecoration(
        color: AppTheme.surfaceColor,
        border: Border(
          top: BorderSide(color: AppTheme.dividerColor),
        ),
      ),
      child: SafeArea(
        child: Row(
          children: [
            Expanded(
              child: TextField(
                controller: _messageController,
                decoration: InputDecoration(
                  hintText: 'Type a message...',
                  filled: true,
                  fillColor: AppTheme.backgroundColor,
                  contentPadding: const EdgeInsets.symmetric(
                    horizontal: 16,
                    vertical: 12,
                  ),
                  border: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(24),
                    borderSide: BorderSide.none,
                  ),
                ),
                maxLines: null,
                textCapitalization: TextCapitalization.sentences,
                onSubmitted: (_) => _sendMessage(),
              ),
            ),
            const SizedBox(width: 12),
            BlocBuilder<ChatBloc, ChatState>(
              builder: (context, state) {
                return Container(
                  decoration: const BoxDecoration(
                    color: AppTheme.primaryColor,
                    shape: BoxShape.circle,
                  ),
                  child: IconButton(
                    onPressed: state.isSending ? null : _sendMessage,
                    icon: state.isSending
                        ? const SizedBox(
                            width: 20,
                            height: 20,
                            child: CircularProgressIndicator(
                              strokeWidth: 2,
                              color: Colors.black,
                            ),
                          )
                        : const Icon(Icons.send, color: Colors.black),
                  ),
                );
              },
            ),
          ],
        ),
      ),
    );
  }

  void _sendMessage() {
    final content = _messageController.text.trim();
    if (content.isEmpty) return;

    context.read<ChatBloc>().add(ChatSendMessage(
      address: _contact.address,
      content: content,
    ));

    _messageController.clear();
    _scrollToBottom();
  }

  void _scrollToBottom() {
    Future.delayed(const Duration(milliseconds: 100), () {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 300),
          curve: Curves.easeOut,
        );
      }
    });
  }

  bool _isSameDay(DateTime a, DateTime b) {
    return a.year == b.year && a.month == b.month && a.day == b.day;
  }

  void _copyMessage(Message message) {
    Clipboard.setData(ClipboardData(text: message.content));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Message copied')),
    );
  }

  void _showOptions() {
    showModalBottomSheet(
      context: context,
      builder: (context) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.copy),
              title: const Text('Copy Address'),
              onTap: () {
                Clipboard.setData(ClipboardData(text: _contact.address));
                Navigator.pop(context);
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(content: Text('Address copied')),
                );
              },
            ),
            ListTile(
              leading: const Icon(Icons.delete_outline, color: AppTheme.errorColor),
              title: const Text('Clear Chat', style: TextStyle(color: AppTheme.errorColor)),
              onTap: () {
                context.read<ChatBloc>().add(ChatClearHistory(_contact.address));
                Navigator.pop(context);
              },
            ),
          ],
        ),
      ),
    );
  }
}
