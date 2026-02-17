import 'dart:convert';
import 'package:shared_preferences/shared_preferences.dart';
import '../models/models.dart';

/// Service for local storage
class StorageService {
  static const String _identityKey = 'identity';
  static const String _contactsKey = 'contacts';
  static const String _configKey = 'config';

  SharedPreferences? _prefs;

  /// Initialize storage
  Future<void> initialize() async {
    _prefs = await SharedPreferences.getInstance();
  }

  /// Check if identity exists
  Future<bool> hasIdentity() async {
    await _ensureInitialized();
    return _prefs!.containsKey(_identityKey);
  }

  /// Save identity
  Future<void> saveIdentity(Identity identity) async {
    await _ensureInitialized();
    final data = jsonEncode({
      'onionAddress': identity.onionAddress,
      'publicKey': identity.publicKey,
      'createdAt': identity.createdAt.toIso8601String(),
    });
    await _prefs!.setString(_identityKey, data);
  }

  /// Load identity
  Future<Identity?> loadIdentity() async {
    await _ensureInitialized();
    final data = _prefs!.getString(_identityKey);
    if (data == null) return null;

    try {
      final map = jsonDecode(data);
      return Identity(
        onionAddress: map['onionAddress'],
        publicKey: map['publicKey'],
        createdAt: DateTime.parse(map['createdAt']),
      );
    } catch (e) {
      return null;
    }
  }

  /// Delete identity
  Future<void> deleteIdentity() async {
    await _ensureInitialized();
    await _prefs!.remove(_identityKey);
  }

  /// Save contacts
  Future<void> saveContacts(List<Contact> contacts) async {
    await _ensureInitialized();
    final data = jsonEncode(contacts.map((c) => {
      'address': c.address,
      'nickname': c.nickname,
      'online': c.online,
      'lastSeen': c.lastSeen?.toIso8601String(),
    }).toList());
    await _prefs!.setString(_contactsKey, data);
  }

  /// Load contacts
  Future<List<Contact>> loadContacts() async {
    await _ensureInitialized();
    final data = _prefs!.getString(_contactsKey);
    if (data == null) return [];

    try {
      final list = jsonDecode(data) as List;
      return list.map((c) => Contact(
        address: c['address'],
        nickname: c['nickname'],
        online: c['online'] ?? false,
        lastSeen: c['lastSeen'] != null ? DateTime.parse(c['lastSeen']) : null,
      )).toList();
    } catch (e) {
      return [];
    }
  }

  /// Add a contact
  Future<void> addContact(Contact contact) async {
    final contacts = await loadContacts();
    if (!contacts.any((c) => c.address == contact.address)) {
      contacts.add(contact);
      await saveContacts(contacts);
    }
  }

  /// Remove a contact
  Future<void> removeContact(String address) async {
    final contacts = await loadContacts();
    contacts.removeWhere((c) => c.address == address);
    await saveContacts(contacts);
  }

  /// Save configuration
  Future<void> saveConfig(AppConfig config) async {
    await _ensureInitialized();
    final data = jsonEncode({
      'allowScreenshots': config.allowScreenshots,
      'showNotificationContent': config.showNotificationContent,
      'requireAuth': config.requireAuth,
      'clipboardTimeout': config.clipboardTimeout,
    });
    await _prefs!.setString(_configKey, data);
  }

  /// Load configuration
  Future<AppConfig> loadConfig() async {
    await _ensureInitialized();
    final data = _prefs!.getString(_configKey);
    if (data == null) return AppConfig.defaultConfig();

    try {
      final map = jsonDecode(data);
      return AppConfig(
        allowScreenshots: map['allowScreenshots'] ?? false,
        showNotificationContent: map['showNotificationContent'] ?? false,
        requireAuth: map['requireAuth'] ?? false,
        clipboardTimeout: map['clipboardTimeout'] ?? 30,
      );
    } catch (e) {
      return AppConfig.defaultConfig();
    }
  }

  /// Clear all data (for privacy)
  Future<void> clearAll() async {
    await _ensureInitialized();
    await _prefs!.clear();
  }

  Future<void> _ensureInitialized() async {
    _prefs ??= await SharedPreferences.getInstance();
  }
}

/// App configuration model for UI
class AppConfig {
  final bool allowScreenshots;
  final bool showNotificationContent;
  final bool requireAuth;
  final int clipboardTimeout;

  AppConfig({
    this.allowScreenshots = false,
    this.showNotificationContent = false,
    this.requireAuth = false,
    this.clipboardTimeout = 30,
  });

  factory AppConfig.defaultConfig() {
    return AppConfig(
      allowScreenshots: false,
      showNotificationContent: false,
      requireAuth: false,
      clipboardTimeout: 30,
    );
  }
}
