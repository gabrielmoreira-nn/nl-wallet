import 'package:wallet_core/core.dart';

import '../../../../../environment.dart';
import '../../../../domain/model/navigation/navigation_request.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../uri_repository.dart';

class CoreUriRepository implements UriRepository {
  final TypedWalletCore _walletCore;

  CoreUriRepository(this._walletCore);

  @override
  Future<NavigationRequest> processUri(Uri uri) async {
    if (Environment.mockRepositories) {
      //FIXME: Move this logic behind identifyUri ? (hard because core doesn't support these types)
      if (uri.toString().contains('issue')) return IssuanceNavigationRequest(uri.toString());
      if (uri.toString().contains('sign')) return SignNavigationRequest(uri.toString());
    }
    final uriType = await _walletCore.identifyUri(uri.toString());
    switch (uriType) {
      case IdentifyUriResult.PidIssuance:
        return PidIssuanceNavigationRequest(uri.toString());
      case IdentifyUriResult.Disclosure:
        return DisclosureNavigationRequest(uri.toString());
    }
  }
}
