import 'package:wallet_core/core.dart';

import '../../../domain/model/attribute/data_attribute.dart';

export '../../../domain/model/pid/pid_issuance_status.dart';

abstract class PidRepository {
  Future<String> getPidIssuanceUrl();

  /// Continue the pidIssuance, returns a preview of all the attributes that will be added if the pid is accepted.
  Future<List<DataAttribute>> continuePidIssuance(String uri);

  Future<void> cancelPidIssuance();

  Future<WalletInstructionResult> acceptOfferedPid(String pin);

  Future<void> rejectOfferedPid();
}
