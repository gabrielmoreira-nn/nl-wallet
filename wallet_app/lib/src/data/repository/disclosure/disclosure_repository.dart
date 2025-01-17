import 'package:wallet_core/core.dart' hide StartDisclosureResult;

import '../../../domain/model/disclosure/start_disclosure_result.dart';

export '../../../domain/model/disclosure/start_disclosure_result.dart';

abstract class DisclosureRepository {
  Future<StartDisclosureResult> startDisclosure(String disclosureUri);

  Future<void> cancelDisclosure();

  Future<AcceptDisclosureResult> acceptDisclosure(String pin);
}
