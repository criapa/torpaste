import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import '../models/models.dart';
import '../utils/theme.dart';

class MessageBubble extends StatelessWidget {
  final Message message;
  final VoidCallback? onLongPress;

  const MessageBubble({
    super.key,
    required this.message,
    this.onLongPress,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment:
            message.isOutgoing ? MainAxisAlignment.end : MainAxisAlignment.start,
        children: [
          if (!message.isOutgoing) const SizedBox(width: 40),
          Flexible(
            child: GestureDetector(
              onLongPress: onLongPress,
              child: Container(
                constraints: BoxConstraints(
                  maxWidth: MediaQuery.of(context).size.width * 0.7,
                ),
                padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
                decoration: BoxDecoration(
                  color: message.isOutgoing
                      ? AppTheme.primaryColor
                      : AppTheme.surfaceColor,
                  borderRadius: BorderRadius.only(
                    topLeft: const Radius.circular(16),
                    topRight: const Radius.circular(16),
                    bottomLeft: Radius.circular(message.isOutgoing ? 16 : 4),
                    bottomRight: Radius.circular(message.isOutgoing ? 4 : 16),
                  ),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    // File indicator
                    if (message.type == MessageType.file && message.file != null)
                      _buildFileContent()
                    else
                      // Regular message content
                      Text(
                        message.content,
                        style: TextStyle(
                          color: message.isOutgoing
                              ? Colors.black
                              : AppTheme.textPrimary,
                          fontSize: 15,
                        ),
                      ),

                    const SizedBox(height: 4),

                    // Timestamp and status
                    Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Text(
                          _formatTime(message.timestamp),
                          style: TextStyle(
                            color: message.isOutgoing
                                ? Colors.black.withOpacity(0.6)
                                : AppTheme.textSecondary,
                            fontSize: 11,
                          ),
                        ),
                        if (message.isOutgoing) ...[
                          const SizedBox(width: 4),
                          _buildStatusIcon(),
                        ],
                      ],
                    ),
                  ],
                ),
              ),
            ),
          ),
          if (message.isOutgoing) const SizedBox(width: 40),
        ],
      ),
    );
  }

  Widget _buildFileContent() {
    final file = message.file!;
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        const Icon(
          Icons.insert_drive_file,
          size: 20,
          color: AppTheme.primaryColor,
        ),
        const SizedBox(width: 8),
        Flexible(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                file.name,
                style: TextStyle(
                  color: message.isOutgoing ? Colors.black : AppTheme.textPrimary,
                  fontWeight: FontWeight.w500,
                ),
                overflow: TextOverflow.ellipsis,
              ),
              Text(
                _formatFileSize(file.size),
                style: TextStyle(
                  color: message.isOutgoing
                      ? Colors.black.withOpacity(0.6)
                      : AppTheme.textSecondary,
                  fontSize: 11,
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildStatusIcon() {
    IconData icon;
    Color color;

    switch (message.status) {
      case MessageStatus.sending:
        icon = Icons.access_time;
        color = Colors.black.withOpacity(0.6);
        break;
      case MessageStatus.sent:
        icon = Icons.check;
        color = Colors.black.withOpacity(0.6);
        break;
      case MessageStatus.delivered:
        icon = Icons.done_all;
        color = Colors.black.withOpacity(0.6);
        break;
      case MessageStatus.read:
        icon = Icons.done_all;
        color = Colors.black;
        break;
      case MessageStatus.failed:
        icon = Icons.error_outline;
        color = AppTheme.errorColor;
        break;
    }

    return Icon(icon, size: 14, color: color);
  }

  String _formatTime(DateTime time) {
    return DateFormat('HH:mm').format(time);
  }

  String _formatFileSize(int bytes) {
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    if (bytes < 1024 * 1024 * 1024) {
      return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
    }
    return '${(bytes / (1024 * 1024 * 1024)).toStringAsFixed(1)} GB';
  }
}
