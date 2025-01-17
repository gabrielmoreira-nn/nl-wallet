import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/disclosure/page/disclosure_confirm_data_attributes_page.dart';
import 'package:wallet/src/util/extension/string_extension.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/mock_data.dart';
import '../../../util/test_utils.dart';

void main() {
  testWidgets('card titles are shown', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      DisclosureConfirmDataAttributesPage(
        onDeclinePressed: () {},
        onAcceptPressed: () {},
        relyingParty: WalletMockData.organization,
        requestedAttributes: {
          WalletMockData.card: WalletMockData.card.attributes,
          WalletMockData.altCard: WalletMockData.altCard.attributes,
        },
        policy: WalletMockData.policy,
        requestPurpose: 'data purpose'.untranslated,
      ),
    );

    // Check if the card title is shown
    final cardFinder = find.textContaining(WalletMockData.card.front.title.testValue);
    expect(cardFinder, findsOneWidget);
    // Check if the altCard title is shown
    final altCardFinder = find.textContaining(WalletMockData.altCard.front.title.testValue);
    expect(altCardFinder, findsOneWidget);
  });

  testWidgets('organization title is shown', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      DisclosureConfirmDataAttributesPage(
        onDeclinePressed: () {},
        onAcceptPressed: () {},
        relyingParty: WalletMockData.organization,
        requestedAttributes: {WalletMockData.card: WalletMockData.card.attributes},
        policy: WalletMockData.policy,
        requestPurpose: 'data purpose'.untranslated,
      ),
    );

    // Check if the card title is shown
    final titleFinder = find.textContaining(WalletMockData.organization.displayName.testValue);
    expect(titleFinder, findsOneWidget);
  });

  testWidgets('data purpose is shown', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      DisclosureConfirmDataAttributesPage(
        onDeclinePressed: () {},
        onAcceptPressed: () {},
        relyingParty: WalletMockData.organization,
        requestedAttributes: {WalletMockData.card: WalletMockData.card.attributes},
        policy: WalletMockData.policy,
        requestPurpose: 'data purpose'.untranslated,
      ),
    );

    // Check if the purpose is shown
    final titleFinder = find.text('data purpose');
    expect(titleFinder, findsOneWidget);
  });

  testWidgets('verify decline button callback', (tester) async {
    bool isCalled = false;
    await tester.pumpWidgetWithAppWrapper(
      DisclosureConfirmDataAttributesPage(
        onDeclinePressed: () => isCalled = true,
        onAcceptPressed: () {},
        relyingParty: WalletMockData.organization,
        requestedAttributes: {WalletMockData.card: WalletMockData.card.attributes},
        policy: WalletMockData.policy,
        requestPurpose: 'data purpose'.untranslated,
      ),
    );

    // Check if the deny listener is triggered correctly
    final l10n = await TestUtils.englishLocalizations;
    final declineButtonFinder = find.text(l10n.disclosureConfirmDataAttributesPageDenyCta, skipOffstage: false);
    // Scroll the button into the viewport so it can be tapped
    await tester.scrollUntilVisible(declineButtonFinder, 50);
    await tester.pumpAndSettle();
    await tester.tap(declineButtonFinder);
    expect(isCalled, isTrue);
  });

  testWidgets('verify accept button callback', (tester) async {
    bool isCalled = false;
    await tester.pumpWidgetWithAppWrapper(
      DisclosureConfirmDataAttributesPage(
        onDeclinePressed: () {},
        onAcceptPressed: () => isCalled = true,
        relyingParty: WalletMockData.organization,
        requestedAttributes: {WalletMockData.card: WalletMockData.card.attributes},
        policy: WalletMockData.policy,
        requestPurpose: 'data purpose'.untranslated,
      ),
    );

    // Check if accept listener is triggered correctly
    final l10n = await TestUtils.englishLocalizations;
    final acceptButtonFinder = find.text(l10n.disclosureConfirmDataAttributesPageApproveCta, skipOffstage: false);
    // Scroll the button into the viewport so it can be tapped
    await tester.scrollUntilVisible(acceptButtonFinder, 50);
    await tester.pumpAndSettle();
    await tester.tap(acceptButtonFinder);
    expect(isCalled, isTrue);
  });
}
