import 'dart:convert';

import 'package:fimber/fimber.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:wallet_core/core.dart';

import '../../../../../environment.dart';
import '../../../../domain/model/navigation/navigation_request.dart';
import '../../../../domain/model/qr/edi_qr_code.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../qr_repository.dart';

class CoreQrRepository implements QrRepository {
  final TypedWalletCore _walletCore;

  CoreQrRepository(this._walletCore);

  @override
  Future<NavigationRequest> processBarcode(Barcode barcode) {
    final legacyValue = _legacyQrToDeeplink(barcode);
    return _processRawValue(legacyValue ?? barcode.rawValue!);
  }

  /// Attempt to convert a legacy style json encoded scenario to a deeplink url that we can process normally.
  /// Sample input: {"id":"DRIVING_LICENSE","type":"issue"}
  /// Returns null if the conversion failed.
  String? _legacyQrToDeeplink(Barcode barcode) {
    if (!Environment.mockRepositories) return null;
    try {
      EdiQrCode.fromJson(jsonDecode(barcode.rawValue!));
      // No exception, so create the deeplink uri that we can process normally
      String url = 'walletdebuginteraction://deeplink#${barcode.rawValue}';
      return Uri.parse(url).toString(); // uri encode the content
    } catch (ex) {
      Fimber.e('Failed to extract process as EdiQrCode. Contents: ${barcode.rawValue}');
    }
    return null;
  }

  Future<NavigationRequest> _processRawValue(String rawValue) async {
    if (Environment.mockRepositories) {
      //FIXME: Move this logic behind identifyUri? (hard because core doesn't support these types)
      if (rawValue.contains('issue')) return IssuanceNavigationRequest(rawValue);
      if (rawValue.contains('sign')) return SignNavigationRequest(rawValue);
    }
    final uriType = await _walletCore.identifyUri(rawValue);
    switch (uriType) {
      case IdentifyUriResult.PidIssuance:
        return PidIssuanceNavigationRequest(rawValue);
      case IdentifyUriResult.Disclosure:
        return DisclosureNavigationRequest(rawValue);
    }
  }
}
