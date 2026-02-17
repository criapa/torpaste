import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import '../blocs/tor_bloc.dart';
import '../blocs/contacts_bloc.dart';
import '../models/models.dart';
import '../utils/theme.dart';
import '../widgets/contact_list_item.dart';
import '../widgets/tor_status_indicator.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('TorChat-Paste'),
        actions: [
          IconButton(
            icon: const Icon(Icons.person_outline),
            onPressed: () => Navigator.pushNamed(context, '/identity'),
            tooltip: 'My Identity',
          ),
          IconButton(
            icon: const Icon(Icons.settings_outlined),
            onPressed: () => Navigator.pushNamed(context, '/settings'),
            tooltip: 'Settings',
          ),
        ],
      ),
      body: Column(
        children: [
          // Tor status indicator
          const TorStatusIndicator(),

          // Contacts list
          Expanded(
            child: BlocBuilder<ContactsBloc, ContactsState>(
              builder: (context, state) {
                if (state.isLoading) {
                  return const Center(
                    child: CircularProgressIndicator(
                      color: AppTheme.primaryColor,
                    ),
                  );
                }

                if (state.contacts.isEmpty) {
                  return _buildEmptyState();
                }

                return ListView.builder(
                  padding: const EdgeInsets.symmetric(vertical: 8),
                  itemCount: state.contacts.length,
                  itemBuilder: (context, index) {
                    final contact = state.contacts[index];
                    return ContactListItem(
                      contact: contact,
                      onTap: () => _openChat(contact),
                      onLongPress: () => _showContactOptions(contact),
                    );
                  },
                );
              },
            ),
          ),
        ],
      ),
      floatingActionButton: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          FloatingActionButton.small(
            heroTag: 'add_contact',
            onPressed: () => Navigator.pushNamed(context, '/add-contact'),
            backgroundColor: AppTheme.surfaceColor,
            child: const Icon(Icons.person_add_outlined, color: AppTheme.primaryColor),
          ),
          const SizedBox(height: 12),
          FloatingActionButton(
            heroTag: 'share_address',
            onPressed: _shareAddress,
            child: const Icon(Icons.share),
          ),
        ],
      ),
    );
  }

  Widget _buildEmptyState() {
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
            const Text(
              'No contacts yet',
              style: TextStyle(
                fontSize: 18,
                fontWeight: FontWeight.w500,
                color: AppTheme.textPrimary,
              ),
            ),
            const SizedBox(height: 8),
            const Text(
              'Add a contact by sharing your address\nor importing from a paste link',
              textAlign: TextAlign.center,
              style: TextStyle(
                fontSize: 14,
                color: AppTheme.textSecondary,
              ),
            ),
            const SizedBox(height: 24),
            OutlinedButton.icon(
              onPressed: () => Navigator.pushNamed(context, '/add-contact'),
              icon: const Icon(Icons.add),
              label: const Text('Add Contact'),
            ),
          ],
        ),
      ),
    );
  }

  void _openChat(Contact contact) {
    Navigator.pushNamed(
      context,
      '/chat',
      arguments: contact,
    );
  }

  void _showContactOptions(Contact contact) {
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
                Clipboard.setData(ClipboardData(text: contact.address));
                Navigator.pop(context);
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(content: Text('Address copied to clipboard')),
                );
              },
            ),
            ListTile(
              leading: const Icon(Icons.delete_outline, color: AppTheme.errorColor),
              title: const Text('Remove Contact', style: TextStyle(color: AppTheme.errorColor)),
              onTap: () {
                context.read<ContactsBloc>().add(ContactsRemove(contact.address));
                Navigator.pop(context);
              },
            ),
          ],
        ),
      ),
    );
  }

  void _shareAddress() async {
    // Show share address dialog
    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (context) => const AlertDialog(
        title: Text('Share Your Address'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            CircularProgressIndicator(color: AppTheme.primaryColor),
            SizedBox(height: 16),
            Text('Creating secure paste...'),
          ],
        ),
      ),
    );

    // Share address
    context.read<ContactsBloc>().add(const ContactsShareAddress());

    // Wait for result
    await Future.delayed(const Duration(seconds: 2));

    if (!mounted) return;
    Navigator.pop(context); // Close loading dialog

    // Show result
    final state = context.read<ContactsBloc>().state;
    if (state.shareUrl != null) {
      _showShareResult(state.shareUrl!);
    } else {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(state.error ?? 'Failed to create paste'),
          backgroundColor: AppTheme.errorColor,
        ),
      );
    }
  }

  void _showShareResult(String url) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Your Address'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Share this link with your contact:',
              style: TextStyle(color: AppTheme.textSecondary),
            ),
            const SizedBox(height: 12),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: AppTheme.backgroundColor,
                borderRadius: BorderRadius.circular(8),
              ),
              child: SelectableText(
                url,
                style: const TextStyle(
                  fontFamily: 'RobotoMono',
                  fontSize: 12,
                ),
              ),
            ),
            const SizedBox(height: 12),
            const Text(
              'This link will expire in 10 minutes.',
              style: TextStyle(
                fontSize: 12,
                color: AppTheme.textSecondary,
              ),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Close'),
          ),
          ElevatedButton.icon(
            onPressed: () {
              Clipboard.setData(ClipboardData(text: url));
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Link copied!')),
              );
              Navigator.pop(context);
            },
            icon: const Icon(Icons.copy, size: 18),
            label: const Text('Copy'),
          ),
        ],
      ),
    );
  }
}
