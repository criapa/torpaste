import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:equatable/equatable.dart';
import '../services/storage_service.dart';
import '../services/pastebin_service.dart';
import '../models/models.dart';

// Events
abstract class ContactsEvent extends Equatable {
  const ContactsEvent();

  @override
  List<Object?> get props => [];
}

class ContactsLoad extends ContactsEvent {}

class ContactsAdd extends ContactsEvent {
  final String address;
  final String? nickname;

  const ContactsAdd({required this.address, this.nickname});

  @override
  List<Object?> get props => [address, nickname];
}

class ContactsRemove extends ContactsEvent {
  final String address;

  const ContactsRemove(this.address);

  @override
  List<Object?> get props => [address];
}

class ContactsImportFromPaste extends ContactsEvent {
  final String pasteUrl;

  const ContactsImportFromPaste(this.pasteUrl);

  @override
  List<Object?> get props => [pasteUrl];
}

class ContactsShareAddress extends ContactsEvent {
  final String? expiration;

  const ContactsShareAddress({this.expiration});

  @override
  List<Object?> get props => [expiration];
}

// State
class ContactsState extends Equatable {
  final List<Contact> contacts;
  final bool isLoading;
  final String? error;
  final String? shareUrl;
  final bool isSharing;

  const ContactsState({
    this.contacts = const [],
    this.isLoading = false,
    this.error,
    this.shareUrl,
    this.isSharing = false,
  });

  ContactsState copyWith({
    List<Contact>? contacts,
    bool? isLoading,
    String? error,
    String? shareUrl,
    bool? isSharing,
  }) {
    return ContactsState(
      contacts: contacts ?? this.contacts,
      isLoading: isLoading ?? this.isLoading,
      error: error,
      shareUrl: shareUrl,
      isSharing: isSharing ?? this.isSharing,
    );
  }

  @override
  List<Object?> get props => [contacts, isLoading, error, shareUrl, isSharing];
}

// BLoC
class ContactsBloc extends Bloc<ContactsEvent, ContactsState> {
  final StorageService storageService;
  final PastebinService pastebinService;

  ContactsBloc({
    required this.storageService,
    required this.pastebinService,
  }) : super(const ContactsState()) {
    on<ContactsLoad>(_onLoad);
    on<ContactsAdd>(_onAdd);
    on<ContactsRemove>(_onRemove);
    on<ContactsImportFromPaste>(_onImportFromPaste);
    on<ContactsShareAddress>(_onShareAddress);
  }

  Future<void> _onLoad(ContactsLoad event, Emitter<ContactsState> emit) async {
    emit(state.copyWith(isLoading: true));

    try {
      final contacts = await storageService.loadContacts();
      emit(state.copyWith(contacts: contacts, isLoading: false));
    } catch (e) {
      emit(state.copyWith(error: e.toString(), isLoading: false));
    }
  }

  Future<void> _onAdd(ContactsAdd event, Emitter<ContactsState> emit) async {
    // Validate address
    if (!_isValidOnionAddress(event.address)) {
      emit(state.copyWith(error: 'Invalid onion address format'));
      return;
    }

    final nickname = event.nickname ?? 'Contact ${state.contacts.length + 1}';
    final contact = Contact(address: event.address, nickname: nickname);

    try {
      await storageService.addContact(contact);
      final contacts = List<Contact>.from(state.contacts)..add(contact);
      emit(state.copyWith(contacts: contacts, error: null));
    } catch (e) {
      emit(state.copyWith(error: e.toString()));
    }
  }

  Future<void> _onRemove(ContactsRemove event, Emitter<ContactsState> emit) async {
    try {
      await storageService.removeContact(event.address);
      final contacts = state.contacts.where((c) => c.address != event.address).toList();
      emit(state.copyWith(contacts: contacts));
    } catch (e) {
      emit(state.copyWith(error: e.toString()));
    }
  }

  Future<void> _onImportFromPaste(ContactsImportFromPaste event, Emitter<ContactsState> emit) async {
    emit(state.copyWith(isLoading: true));

    try {
      // Fetch paste content
      final content = await pastebinService.fetchPaste(event.pasteUrl);
      if (content == null) {
        emit(state.copyWith(isLoading: false, error: 'Failed to fetch paste'));
        return;
      }

      // Extract onion address
      final address = pastebinService.extractOnionAddress(content);
      if (address == null) {
        emit(state.copyWith(isLoading: false, error: 'No valid onion address found'));
        return;
      }

      // Add contact
      add(ContactsAdd(address: address));
      emit(state.copyWith(isLoading: false));
    } catch (e) {
      emit(state.copyWith(isLoading: false, error: e.toString()));
    }
  }

  Future<void> _onShareAddress(ContactsShareAddress event, Emitter<ContactsState> emit) async {
    emit(state.copyWith(isSharing: true));

    try {
      // Get address from storage
      final identity = await storageService.loadIdentity();
      if (identity == null) {
        emit(state.copyWith(isSharing: false, error: 'No identity found'));
        return;
      }

      // Create paste
      final result = await pastebinService.createPaste(
        content: identity.onionAddress,
        expiration: event.expiration ?? '10M',
        private: true,
      );

      if (result != null) {
        emit(state.copyWith(isSharing: false, shareUrl: result.url));
      } else {
        emit(state.copyWith(isSharing: false, error: 'Failed to create paste'));
      }
    } catch (e) {
      emit(state.copyWith(isSharing: false, error: e.toString()));
    }
  }

  bool _isValidOnionAddress(String address) {
    final addr = address.replaceAll('.onion', '');
    return addr.length == 56 && RegExp(r'^[a-z2-7]+$').hasMatch(addr);
  }
}
