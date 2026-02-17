import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import '../blocs/tor_bloc.dart';
import '../models/models.dart';
import '../utils/theme.dart';

class TorStatusIndicator extends StatelessWidget {
  const TorStatusIndicator({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<TorBloc, TorState>(
      builder: (context, state) {
        return Container(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
          decoration: const BoxDecoration(
            color: AppTheme.surfaceColor,
            border: Border(
              bottom: BorderSide(color: AppTheme.dividerColor),
            ),
          ),
          child: Row(
            children: [
              // Status icon
              _buildStatusIcon(state.status),
              const SizedBox(width: 12),

              // Status text
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      _getStatusText(state.status),
                      style: TextStyle(
                        fontWeight: FontWeight.w500,
                        color: _getStatusColor(state.status),
                      ),
                    ),
                    if (state.status == TorConnectionStatus.connecting)
                      Padding(
                        padding: const EdgeInsets.only(top: 4),
                        child: Row(
                          children: [
                            Expanded(
                              child: ClipRRect(
                                borderRadius: BorderRadius.circular(2),
                                child: LinearProgressIndicator(
                                  value: state.bootstrapProgress / 100,
                                  backgroundColor: AppTheme.dividerColor,
                                  valueColor: AlwaysStoppedAnimation<Color>(
                                    AppTheme.primaryColor,
                                  ),
                                  minHeight: 3,
                                ),
                              ),
                            ),
                            const SizedBox(width: 8),
                            Text(
                              '${state.bootstrapProgress}%',
                              style: const TextStyle(
                                fontSize: 11,
                                color: AppTheme.textSecondary,
                              ),
                            ),
                          ],
                        ),
                      ),
                    if (state.status == TorConnectionStatus.ready &&
                        state.onionAddress != null)
                      Text(
                        state.onionAddress!.substring(0, 20) + '...',
                        style: const TextStyle(
                          fontSize: 11,
                          color: AppTheme.textSecondary,
                          fontFamily: 'RobotoMono',
                        ),
                      ),
                  ],
                ),
              ),

              // Connection indicator
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: _getStatusColor(state.status).withOpacity(0.1),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Container(
                      width: 6,
                      height: 6,
                      decoration: BoxDecoration(
                        color: _getStatusColor(state.status),
                        shape: BoxShape.circle,
                      ),
                    ),
                    const SizedBox(width: 6),
                    Text(
                      _getConnectionText(state.status),
                      style: TextStyle(
                        fontSize: 11,
                        fontWeight: FontWeight.w500,
                        color: _getStatusColor(state.status),
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildStatusIcon(TorConnectionStatus status) {
    switch (status) {
      case TorConnectionStatus.disconnected:
        return const Icon(
          Icons.signal_wifi_off,
          size: 20,
          color: AppTheme.textSecondary,
        );
      case TorConnectionStatus.connecting:
        return const SizedBox(
          width: 20,
          height: 20,
          child: CircularProgressIndicator(
            strokeWidth: 2,
            color: AppTheme.primaryColor,
          ),
        );
      case TorConnectionStatus.ready:
        return const Icon(
          Icons.signal_wifi_4_bar,
          size: 20,
          color: AppTheme.primaryColor,
        );
      case TorConnectionStatus.error:
        return const Icon(
          Icons.error_outline,
          size: 20,
          color: AppTheme.errorColor,
        );
    }
  }

  String _getStatusText(TorConnectionStatus status) {
    switch (status) {
      case TorConnectionStatus.disconnected:
        return 'Disconnected';
      case TorConnectionStatus.connecting:
        return 'Connecting to Tor network...';
      case TorConnectionStatus.ready:
        return 'Connected to Tor';
      case TorConnectionStatus.error:
        return 'Connection error';
    }
  }

  String _getConnectionText(TorConnectionStatus status) {
    switch (status) {
      case TorConnectionStatus.disconnected:
        return 'OFF';
      case TorConnectionStatus.connecting:
        return '...';
      case TorConnectionStatus.ready:
        return 'ON';
      case TorConnectionStatus.error:
        return 'ERR';
    }
  }

  Color _getStatusColor(TorConnectionStatus status) {
    switch (status) {
      case TorConnectionStatus.disconnected:
        return AppTheme.textSecondary;
      case TorConnectionStatus.connecting:
        return AppTheme.primaryColor;
      case TorConnectionStatus.ready:
        return AppTheme.primaryColor;
      case TorConnectionStatus.error:
        return AppTheme.errorColor;
    }
  }
}
