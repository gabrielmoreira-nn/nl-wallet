import 'dart:async';

import 'package:collection/collection.dart';
import 'package:wallet_core/core.dart';

import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../pid_repository.dart';

class CorePidRepository extends PidRepository {
  final TypedWalletCore _walletCore;
  final Mapper<Card, WalletCard> _cardMapper;

  CorePidRepository(
    this._walletCore,
    this._cardMapper,
  );

  @override
  Future<String> getPidIssuanceUrl() => _walletCore.createPidIssuanceRedirectUri();

  @override
  Future<List<DataAttribute>> continuePidIssuance(String uri) async {
    final result = await _walletCore.continuePidIssuance(uri);
    return result.map((card) => _cardMapper.map(card)).map((card) => card.attributes).flattened.toList();
  }

  @override
  Future<void> cancelPidIssuance() => _walletCore.cancelPidIssuance();

  @override
  Future<WalletInstructionResult> acceptOfferedPid(String pin) => _walletCore.acceptOfferedPid(pin);

  @override
  Future<void> rejectOfferedPid() => _walletCore.rejectOfferedPid();
}
