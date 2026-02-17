import 'dart:async';
import 'dart:convert';
import 'package:http/http.dart' as http;
import 'dart:math';

/// Service for Pastebin integration
class PastebinService {
  static const String _baseUrl = 'https://pastebin.com';
  static const String _apiUrl = 'https://pastebin.com/api/api_post.php';

  // Users should get their own API key from pastebin.com
  String _apiKey = 'qk07ECs684p384tqyrBPGYUa_fCJkxlk';

  /// Set the API key
  void setApiKey(String key) {
    _apiKey = key;
  }

  /// Create a paste with an onion address
  Future<PasteResult?> createPaste({
    required String content,
    String expiration = '10M',
    bool private = true,
  }) async {
    try {
      // For demo purposes, simulate paste creation
      // In production, use actual API
      return _simulatePasteCreation(content);
    } catch (e) {
      return null;
    }
  }

  /// Simulate paste creation for demo
  PasteResult _simulatePasteCreation(String content) {
    // Generate a random paste key
    const chars = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
    final random = Random.secure();
    final key = List.generate(8, (_) => chars[random.nextInt(chars.length)]).join();

    return PasteResult(
      url: '$_baseUrl/$key',
      rawUrl: '$_baseUrl/raw/$key',
      expires: DateTime.now().add(const Duration(minutes: 10)).toIso8601String(),
    );
  }

  /// Fetch content from a paste URL
  Future<String?> fetchPaste(String url) async {
    try {
      // Parse URL to get raw content
      final uri = Uri.parse(url);

      // Extract paste key
      String path = uri.path;
      if (path.startsWith('/raw/')) {
        path = path.substring(5);
      } else if (path.startsWith('/')) {
        path = path.substring(1);
      }

      // Remove trailing slashes
      path = path.replaceAll('/', '');

      if (path.isEmpty || path.length > 20) {
        return null;
      }

      // For demo, return a simulated onion address
      // In production, make actual HTTP request
      return _simulateFetchPaste(path);
    } catch (e) {
      return null;
    }
  }

  /// Simulate fetching a paste
  String? _simulateFetchPaste(String key) {
    // Generate a valid-looking onion address
    const chars = 'abcdefghijklmnopqrstuvwxyz234567';
    final random = Random.secure();
    final address = List.generate(56, (_) => chars[random.nextInt(chars.length)]).join();
    return '$address.onion';
  }

  /// Extract onion address from paste content
  String? extractOnionAddress(String content) {
    final trimmed = content.trim();

    // Try to find .onion address
    final match = RegExp(r'([a-z2-7]{56}\.onion)').firstMatch(trimmed);
    if (match != null) {
      return match.group(1);
    }

    // If no .onion suffix, check if it's just the address
    if (RegExp(r'^[a-z2-7]{56}$').hasMatch(trimmed)) {
      return '$trimmed.onion';
    }

    return null;
  }

  /// Validate paste URL
  bool isValidPasteUrl(String url) {
    try {
      final uri = Uri.parse(url);
      return uri.host.contains('pastebin.com') ||
             uri.host.contains('pastebin');
    } catch (e) {
      return false;
    }
  }
}

/// Result from pastebin API
class PasteResult {
  final String url;
  final String rawUrl;
  final String? expires;

  PasteResult({
    required this.url,
    required this.rawUrl,
    this.expires,
  });
}
