import 'package:flutter/material.dart';
import '../utils/theme.dart';

class SettingsScreen extends StatefulWidget {
  const SettingsScreen({super.key});

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  bool _allowScreenshots = false;
  bool _showNotifications = false;
  bool _requireAuth = false;
  int _clipboardTimeout = 30;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Settings'),
      ),
      body: ListView(
        children: [
          _buildSectionHeader('Security'),
          SwitchListTile(
            title: const Text('Allow Screenshots'),
            subtitle: const Text(
              'Disabling is recommended for privacy',
              style: TextStyle(fontSize: 12),
            ),
            value: _allowScreenshots,
            onChanged: (value) => setState(() => _allowScreenshots = value),
            activeColor: AppTheme.primaryColor,
          ),
          SwitchListTile(
            title: const Text('Notification Content'),
            subtitle: const Text(
              'Show message content in notifications',
              style: TextStyle(fontSize: 12),
            ),
            value: _showNotifications,
            onChanged: (value) => setState(() => _showNotifications = value),
            activeColor: AppTheme.primaryColor,
          ),
          SwitchListTile(
            title: const Text('Require Authentication'),
            subtitle: const Text(
              'Require auth on app open',
              style: TextStyle(fontSize: 12),
            ),
            value: _requireAuth,
            onChanged: (value) => setState(() => _requireAuth = value),
            activeColor: AppTheme.primaryColor,
          ),
          const Divider(),

          _buildSectionHeader('Privacy'),
          ListTile(
            title: const Text('Clear Clipboard'),
            subtitle: Text(
              'After $_clipboardTimeout seconds',
              style: const TextStyle(fontSize: 12),
            ),
            trailing: DropdownButton<int>(
              value: _clipboardTimeout,
              underline: const SizedBox(),
              items: [0, 15, 30, 60, 120].map((seconds) {
                return DropdownMenuItem(
                  value: seconds,
                  child: Text(
                    seconds == 0 ? 'Never' : '${seconds}s',
                    style: const TextStyle(color: AppTheme.textPrimary),
                  ),
                );
              }).toList(),
              onChanged: (value) {
                if (value != null) {
                  setState(() => _clipboardTimeout = value);
                }
              },
            ),
          ),
          const Divider(),

          _buildSectionHeader('Network'),
          const ListTile(
            title: Text('Connection'),
            subtitle: Text(
              'Using Tor network (SOCKS5)',
              style: TextStyle(fontSize: 12),
            ),
            leading: Icon(Icons.lock_outline, color: AppTheme.primaryColor),
          ),
          const Divider(),

          _buildSectionHeader('About'),
          ListTile(
            title: const Text('Version'),
            trailing: const Text(
              '0.1.0',
              style: TextStyle(color: AppTheme.textSecondary),
            ),
          ),
          ListTile(
            title: const Text('Open Source'),
            subtitle: const Text(
              'View source code',
              style: TextStyle(fontSize: 12),
            ),
            trailing: const Icon(Icons.open_in_new, size: 18),
            onTap: () {
              // Open source link
            },
          ),

          const SizedBox(height: 32),

          // Danger zone
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Danger Zone',
                  style: TextStyle(
                    color: AppTheme.errorColor,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 12),
                SizedBox(
                  width: double.infinity,
                  child: OutlinedButton.icon(
                    onPressed: _showWipeDialog,
                    icon: const Icon(Icons.delete_forever, color: AppTheme.errorColor),
                    label: const Text(
                      'Wipe All Data',
                      style: TextStyle(color: AppTheme.errorColor),
                    ),
                    style: OutlinedButton.styleFrom(
                      side: const BorderSide(color: AppTheme.errorColor),
                    ),
                  ),
                ),
              ],
            ),
          ),

          const SizedBox(height: 32),
        ],
      ),
    );
  }

  Widget _buildSectionHeader(String title) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
      child: Text(
        title,
        style: const TextStyle(
          color: AppTheme.primaryColor,
          fontWeight: FontWeight.w600,
          fontSize: 14,
        ),
      ),
    );
  }

  void _showWipeDialog() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Wipe All Data?'),
        content: const Text(
          'This will permanently delete:\n\n'
          '• Your identity and keys\n'
          '• All contacts\n'
          '• All chat history\n\n'
          'This action cannot be undone.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              // Wipe all data
              Navigator.pop(context);
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(
                  content: Text('All data wiped'),
                  backgroundColor: AppTheme.errorColor,
                ),
              );
            },
            style: ElevatedButton.styleFrom(
              backgroundColor: AppTheme.errorColor,
            ),
            child: const Text('Wipe'),
          ),
        ],
      ),
    );
  }
}
