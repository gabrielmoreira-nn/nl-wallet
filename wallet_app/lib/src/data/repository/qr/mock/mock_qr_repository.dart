import 'dart:convert';

import '../../../../domain/model/qr/edi_qr_code.dart';
import '../../../../domain/model/qr/qr_request.dart';
import '../qr_repository.dart';

class MockQrRepository implements QrRepository {
  MockQrRepository();

  @override
  Future<QrRequest> getRequest(rawValue) async {
    final json = jsonDecode(rawValue);
    final code = EdiQrCode.fromJson(json);
    switch (code.type) {
      case EdiQrType.issuance:
        return QrIssuanceRequest(code.id);
      case EdiQrType.disclosure:
        return QrDisclosureRequest(code.id);
      case EdiQrType.sign:
        return QrSignRequest(code.id);
    }
  }
}
