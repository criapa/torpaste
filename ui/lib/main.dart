import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'screens/splash_screen.dart';
import 'screens/home_screen.dart';
import 'screens/chat_screen.dart';
import 'screens/settings_screen.dart';
import 'screens/identity_screen.dart';
import 'screens/add_contact_screen.dart';
import 'services/tor_service.dart';
import 'services/chat_service.dart';
import 'services/pastebin_service.dart';
import 'services/storage_service.dart';
import 'blocs/tor_bloc.dart';
import 'blocs/chat_bloc.dart';
import 'blocs/contacts_bloc.dart';
import 'utils/theme.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();

  // Set preferred orientations
  SystemChrome.setPreferredOrientations([
    DeviceOrientation.portraitUp,
    DeviceOrientation.portraitDown,
  ]);

  // Set system UI overlay style
  SystemChrome.setSystemUIOverlayStyle(
    const SystemUiOverlayStyle(
      statusBarColor: Colors.transparent,
      statusBarIconBrightness: Brightness.light,
      systemNavigationBarColor: Color(0xFF121212),
      systemNavigationBarIconBrightness: Brightness.light,
    ),
  );

  runApp(const TorChatPasteApp());
}

class TorChatPasteApp extends StatelessWidget {
  const TorChatPasteApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiRepositoryProvider(
      providers: [
        RepositoryProvider<TorService>(
          create: (_) => TorService(),
        ),
        RepositoryProvider<ChatService>(
          create: (_) => ChatService(),
        ),
        RepositoryProvider<PastebinService>(
          create: (_) => PastebinService(),
        ),
        RepositoryProvider<StorageService>(
          create: (_) => StorageService(),
        ),
      ],
      child: MultiBlocProvider(
        providers: [
          BlocProvider<TorBloc>(
            create: (context) => TorBloc(
              torService: context.read<TorService>(),
              storageService: context.read<StorageService>(),
            )..add(TorInitialize()),
          ),
          BlocProvider<ContactsBloc>(
            create: (context) => ContactsBloc(
              storageService: context.read<StorageService>(),
              pastebinService: context.read<PastebinService>(),
            ),
          ),
          BlocProvider<ChatBloc>(
            create: (context) => ChatBloc(
              chatService: context.read<ChatService>(),
            ),
          ),
        ],
        child: MaterialApp(
          title: 'TorChat-Paste',
          debugShowCheckedModeBanner: false,
          theme: AppTheme.darkTheme,
          home: const SplashScreen(),
          routes: {
            '/home': (context) => const HomeScreen(),
            '/chat': (context) => const ChatScreen(),
            '/settings': (context) => const SettingsScreen(),
            '/identity': (context) => const IdentityScreen(),
            '/add-contact': (context) => const AddContactScreen(),
          },
        ),
      ),
    );
  }
}
