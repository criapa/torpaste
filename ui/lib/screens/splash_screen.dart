import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import '../blocs/tor_bloc.dart';
import '../blocs/contacts_bloc.dart';
import '../utils/theme.dart';

class SplashScreen extends StatefulWidget {
  const SplashScreen({super.key});

  @override
  State<SplashScreen> createState() => _SplashScreenState();
}

class _SplashScreenState extends State<SplashScreen> {
  @override
  void initState() {
    super.initState();
    _initializeApp();
  }

  Future<void> _initializeApp() async {
    // Wait for Tor to initialize
    await Future.delayed(const Duration(seconds: 3));

    if (mounted) {
      // Load contacts
      context.read<ContactsBloc>().add(ContactsLoad());

      // Navigate to home
      Navigator.of(context).pushReplacementNamed('/home');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: BlocBuilder<TorBloc, TorState>(
        builder: (context, state) {
          return Container(
            decoration: const BoxDecoration(
              color: AppTheme.backgroundColor,
            ),
            child: SafeArea(
              child: Center(
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    // Logo
                    Container(
                      width: 120,
                      height: 120,
                      decoration: BoxDecoration(
                        color: AppTheme.primaryColor.withOpacity(0.1),
                        shape: BoxShape.circle,
                      ),
                      child: const Icon(
                        Icons.lock_outline,
                        size: 60,
                        color: AppTheme.primaryColor,
                      ),
                    ),
                    const SizedBox(height: 32),

                    // App name
                    const Text(
                      'TorChat-Paste',
                      style: TextStyle(
                        fontSize: 28,
                        fontWeight: FontWeight.bold,
                        color: AppTheme.textPrimary,
                        fontFamily: 'RobotoMono',
                      ),
                    ),
                    const SizedBox(height: 8),
                    const Text(
                      'Anonymous P2P Messenger',
                      style: TextStyle(
                        fontSize: 14,
                        color: AppTheme.textSecondary,
                      ),
                    ),
                    const SizedBox(height: 48),

                    // Status
                    if (state.status == TorConnectionStatus.connecting) ...[
                      _buildLoadingIndicator(state.bootstrapProgress),
                      const SizedBox(height: 16),
                      Text(
                        'Connecting to Tor network...',
                        style: TextStyle(
                          color: AppTheme.textSecondary,
                          fontSize: 14,
                        ),
                      ),
                    ] else if (state.status == TorConnectionStatus.ready) ...[
                      const Icon(
                        Icons.check_circle,
                        color: AppTheme.primaryColor,
                        size: 48,
                      ),
                      const SizedBox(height: 16),
                      const Text(
                        'Connected',
                        style: TextStyle(
                          color: AppTheme.primaryColor,
                          fontSize: 16,
                        ),
                      ),
                      if (state.onionAddress != null) ...[
                        const SizedBox(height: 8),
                        Text(
                          state.onionAddress!.substring(0, 16) + '...',
                          style: const TextStyle(
                            color: AppTheme.textSecondary,
                            fontSize: 12,
                            fontFamily: 'RobotoMono',
                          ),
                        ),
                      ],
                    ] else if (state.status == TorConnectionStatus.error) ...[
                      const Icon(
                        Icons.error_outline,
                        color: AppTheme.errorColor,
                        size: 48,
                      ),
                      const SizedBox(height: 16),
                      Text(
                        state.errorMessage ?? 'Connection failed',
                        style: const TextStyle(
                          color: AppTheme.errorColor,
                          fontSize: 14,
                        ),
                        textAlign: TextAlign.center,
                      ),
                    ],
                  ],
                ),
              ),
            ),
          );
        },
      ),
    );
  }

  Widget _buildLoadingIndicator(int progress) {
    return Column(
      children: [
        SizedBox(
          width: 200,
          height: 4,
          child: ClipRRect(
            borderRadius: BorderRadius.circular(2),
            child: LinearProgressIndicator(
              value: progress / 100,
              backgroundColor: AppTheme.dividerColor,
              valueColor: const AlwaysStoppedAnimation<Color>(AppTheme.primaryColor),
            ),
          ),
        ),
        const SizedBox(height: 8),
        Text(
          '$progress%',
          style: const TextStyle(
            color: AppTheme.textSecondary,
            fontSize: 12,
          ),
        ),
      ],
    );
  }
}
