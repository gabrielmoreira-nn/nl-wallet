import '../../../domain/model/wallet_card.dart';

abstract class WalletCardRepository {
  Stream<List<WalletCard>> observeWalletCards();

  Future<bool> exists(String docType);

  Future<List<WalletCard>> readAll();

  Future<WalletCard> read(String docType);
}
