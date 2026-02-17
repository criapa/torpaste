import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import '../blocs/contacts_bloc.dart';
import '../utils/theme.dart';

class AddContactScreen extends StatefulWidget {
  const AddContactScreen({super.key});

  @override
  State<AddContactScreen> createState() => _AddContactScreenState();
}

class _AddContactScreenState extends State<AddContactScreen> {
  final _formKey = GlobalKey<FormState>();
  final _pasteUrlController = TextEditingController();
  final _onionAddressController = TextEditingController();
  final _nicknameController = TextEditingController();

  bool _isPasteMode = true;

  @override
  void dispose() {
    _pasteUrlController.dispose();
    _onionAddressController.dispose();
    _nicknameController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Add Contact'),
      ),
      body: BlocListener<ContactsBloc, ContactsState>(
        listener: (context, state) {
          if (state.error != null) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(
                content: Text(state.error!),
                backgroundColor: AppTheme.errorColor,
              ),
            );
          }
        },
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: Form(
            key: _formKey,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Mode selector
                Container(
                  decoration: BoxDecoration(
                    color: AppTheme.surfaceColor,
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Row(
                    children: [
                      Expanded(
                        child: _buildModeButton(
                          'Paste Link',
                          Icons.link,
                          _isPasteMode,
                          () => setState(() => _isPasteMode = true),
                        ),
                      ),
                      Expanded(
                        child: _buildModeButton(
                          'Direct Address',
                          Icons.dns_outlined,
                          !_isPasteMode,
                          () => setState(() => _isPasteMode = false),
                        ),
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 32),

                if (_isPasteMode) ...[
                  // Paste URL input
                  const Text(
                    'Paste URL',
                    style: TextStyle(
                      fontSize: 14,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                  const SizedBox(height: 8),
                  TextFormField(
                    controller: _pasteUrlController,
                    decoration: InputDecoration(
                      hintText: 'https://pastebin.com/abc123',
                      prefixIcon: const Icon(Icons.link),
                      suffixIcon: IconButton(
                        icon: const Icon(Icons.paste),
                        onPressed: _pasteFromClipboard,
                      ),
                    ),
                    keyboardType: TextInputType.url,
                    validator: (value) {
                      if (value == null || value.isEmpty) {
                        return 'Please enter a paste URL';
                      }
                      if (!value.contains('pastebin.com') &&
                          !value.contains('pastebin')) {
                        return 'Please enter a valid pastebin URL';
                      }
                      return null;
                    },
                  ),
                  const SizedBox(height: 12),
                  const Text(
                    'The paste should contain the onion address of your contact.',
                    style: TextStyle(
                      fontSize: 12,
                      color: AppTheme.textSecondary,
                    ),
                  ),
                ] else ...[
                  // Direct onion address input
                  const Text(
                    'Onion Address',
                    style: TextStyle(
                      fontSize: 14,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                  const SizedBox(height: 8),
                  TextFormField(
                    controller: _onionAddressController,
                    decoration: InputDecoration(
                      hintText: 'abcdef...xyz.onion',
                      prefixIcon: const Icon(Icons.dns_outlined),
                      suffixIcon: IconButton(
                        icon: const Icon(Icons.paste),
                        onPressed: _pasteFromClipboard,
                      ),
                    ),
                    style: const TextStyle(
                      fontFamily: 'RobotoMono',
                      fontSize: 13,
                    ),
                    validator: (value) {
                      if (value == null || value.isEmpty) {
                        return 'Please enter an onion address';
                      }
                      final addr = value.replaceAll('.onion', '');
                      if (addr.length != 56) {
                        return 'Address must be 56 characters';
                      }
                      if (!RegExp(r'^[a-z2-7]+$').hasMatch(addr)) {
                        return 'Invalid characters in address';
                      }
                      return null;
                    },
                  ),
                  const SizedBox(height: 12),
                  const Text(
                    'Enter your contact\'s full onion address (v3).',
                    style: TextStyle(
                      fontSize: 12,
                      color: AppTheme.textSecondary,
                    ),
                  ),
                ],

                const SizedBox(height: 24),

                // Nickname
                const Text(
                  'Nickname (Optional)',
                  style: TextStyle(
                    fontSize: 14,
                    fontWeight: FontWeight.w500,
                  ),
                ),
                const SizedBox(height: 8),
                TextFormField(
                  controller: _nicknameController,
                  decoration: const InputDecoration(
                    hintText: 'Enter a nickname for this contact',
                    prefixIcon: Icon(Icons.person_outline),
                  ),
                  textCapitalization: TextCapitalization.words,
                ),

                const SizedBox(height: 40),

                // Add button
                SizedBox(
                  width: double.infinity,
                  child: BlocBuilder<ContactsBloc, ContactsState>(
                    builder: (context, state) {
                      return ElevatedButton(
                        onPressed: state.isLoading ? null : _addContact,
                        child: state.isLoading
                            ? const SizedBox(
                                width: 20,
                                height: 20,
                                child: CircularProgressIndicator(
                                  strokeWidth: 2,
                                  color: Colors.black,
                                ),
                              )
                            : const Text('Add Contact'),
                      );
                    },
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildModeButton(
    String label,
    IconData icon,
    bool isSelected,
    VoidCallback onTap,
  ) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.symmetric(vertical: 16),
        decoration: BoxDecoration(
          color: isSelected ? AppTheme.primaryColor.withOpacity(0.1) : Colors.transparent,
          borderRadius: BorderRadius.circular(12),
          border: isSelected
              ? Border.all(color: AppTheme.primaryColor, width: 2)
              : null,
        ),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              icon,
              size: 20,
              color: isSelected ? AppTheme.primaryColor : AppTheme.textSecondary,
            ),
            const SizedBox(width: 8),
            Text(
              label,
              style: TextStyle(
                color: isSelected ? AppTheme.primaryColor : AppTheme.textSecondary,
                fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
              ),
            ),
          ],
        ),
      ),
    );
  }

  void _pasteFromClipboard() async {
    final data = await Clipboard.getData(Clipboard.kTextPlain);
    if (data?.text != null) {
      if (_isPasteMode) {
        _pasteUrlController.text = data!.text!;
      } else {
        _onionAddressController.text = data!.text!;
      }
    }
  }

  void _addContact() {
    if (!_formKey.currentState!.validate()) return;

    final nickname = _nicknameController.text.trim();

    if (_isPasteMode) {
      context.read<ContactsBloc>().add(
        ContactsImportFromPaste(_pasteUrlController.text.trim()),
      );
    } else {
      final address = _onionAddressController.text.trim();
      context.read<ContactsBloc>().add(
        ContactsAdd(
          address: address.endsWith('.onion') ? address : '$address.onion',
          nickname: nickname.isNotEmpty ? nickname : null,
        ),
      );
    }

    // Wait a moment then check result
    Future.delayed(const Duration(seconds: 2), () {
      if (mounted) {
        final state = context.read<ContactsBloc>().state;
        if (state.contacts.isNotEmpty && state.error == null) {
          Navigator.pop(context);
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('Contact added')),
          );
        }
      }
    });
  }
}
