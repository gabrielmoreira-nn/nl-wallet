import 'attribute/missing_attribute.dart';
import 'organization.dart';
import 'policy/policy.dart';
import 'wallet_card.dart';

class IssuanceResponse {
  final Organization organization;
  final List<MockRequestedAttribute> requestedAttributes;
  final String requestPurpose;
  final Policy policy;
  final List<WalletCard> cards;

  const IssuanceResponse({
    required this.organization,
    required this.requestedAttributes,
    required this.requestPurpose,
    required this.policy,
    required this.cards,
  });
}
