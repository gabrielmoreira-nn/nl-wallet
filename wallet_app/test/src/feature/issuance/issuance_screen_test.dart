import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/multiple_cards_flow.dart';
import 'package:wallet/src/domain/usecase/pin/confirm_transaction_usecase.dart';
import 'package:wallet/src/feature/issuance/bloc/issuance_bloc.dart';
import 'package:wallet/src/feature/issuance/issuance_screen.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/mock_data.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';
import '../pin/pin_page_test.dart';

class MockIssuanceBloc extends MockBloc<IssuanceEvent, IssuanceState> implements IssuanceBloc {}

class MockConfirmTransactionUseCase implements ConfirmTransactionUseCase {
  @override
  Future<CheckPinResult> invoke(String pin) => throw UnimplementedError();
}

void main() {
  MultipleCardsFlow mockMultipleCardsFlow = MultipleCardsFlow(
    cardToOrganizations: {
      WalletMockData.card: WalletMockData.organization,
      WalletMockData.altCard: WalletMockData.organization,
    },
    selectedCardIds: {WalletMockData.card.id},
    activeIndex: 0,
  );

  group('goldens', () {
    testGoldens('IssuanceInitial Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
              MockIssuanceBloc(),
              const IssuanceInitial(isRefreshFlow: false),
            ),
            name: 'initial',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'initial.light');
    });

    testGoldens('IssuanceLoadInProgress Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
              MockIssuanceBloc(),
              const IssuanceLoadInProgress(isRefreshFlow: false),
            ),
            name: 'load_in_progress',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'load_in_progress.light');
    });

    testGoldens('IssuanceCheckOrganization Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
              MockIssuanceBloc(),
              IssuanceCheckOrganization(
                organization: WalletMockData.organization,
                isRefreshFlow: false,
              ),
            ),
            name: 'check_organization',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'check_organization.light');
    });

    testGoldens('IssuanceProofIdentity Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
              MockIssuanceBloc(),
              IssuanceProofIdentity(
                organization: WalletMockData.organization,
                requestedAttributes: [WalletMockData.textDataAttribute],
                policy: WalletMockData.policy,
                isRefreshFlow: false,
              ),
            ),
            name: 'proof_identity',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'proof_identity.light');
    });

    testGoldens('PinEntryInProgress Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: RepositoryProvider<ConfirmTransactionUseCase>.value(
              value: MockConfirmTransactionUseCase(),
              child: const IssuanceScreen()
                  .withState<IssuanceBloc, IssuanceState>(
                    MockIssuanceBloc(),
                    const IssuanceProvidePin(isRefreshFlow: false),
                  )
                  .withState<PinBloc, PinState>(
                    MockPinBloc(),
                    const PinEntryInProgress(0),
                  ),
            ),
            name: 'provide_pin',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'provide_pin.light');
    });

    testGoldens('IssuanceCheckDataOffering Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
              MockIssuanceBloc(),
              IssuanceCheckDataOffering(isRefreshFlow: false, card: WalletMockData.card),
            ),
            name: 'check_data_offering',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'check_data_offering.light');
    });

    testGoldens('IssuanceSelectCards Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
              MockIssuanceBloc(),
              IssuanceSelectCards(
                multipleCardsFlow: mockMultipleCardsFlow,
                isRefreshFlow: false,
              ),
            ),
            name: 'select_cards',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'select_cards.light');
    });

    testGoldens('IssuanceCheckCards Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
              MockIssuanceBloc(),
              IssuanceCheckCards(isRefreshFlow: false, multipleCardsFlow: mockMultipleCardsFlow),
            ),
            name: 'check_cards',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'check_cards.light');
    });

    testGoldens('IssuanceCompleted Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
              MockIssuanceBloc(),
              IssuanceCompleted(isRefreshFlow: false, addedCards: [WalletMockData.card]),
            ),
            name: 'completed',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'completed.light');
    });

    testGoldens('IssuanceStopped Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
              MockIssuanceBloc(),
              const IssuanceStopped(isRefreshFlow: false),
            ),
            name: 'stopped',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'stopped.light');
    });

    testGoldens('IssuanceGenericError Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
              MockIssuanceBloc(),
              const IssuanceGenericError(isRefreshFlow: false),
            ),
            name: 'generic_error',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'generic_error.light');
    });

    testGoldens('IssuanceIdentityValidationFailure Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
              MockIssuanceBloc(),
              const IssuanceIdentityValidationFailure(isRefreshFlow: false),
            ),
            name: 'identity_validation_error',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'identity_validation_error.light');
    });

    testGoldens('IssuanceLoadFailure Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
              MockIssuanceBloc(),
              const IssuanceLoadFailure(isRefreshFlow: false),
            ),
            name: 'load_failure',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'load_failure.light');
    });

    testGoldens('IssuanceCompleted Dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
              MockIssuanceBloc(),
              IssuanceCompleted(isRefreshFlow: false, addedCards: [WalletMockData.card, WalletMockData.altCard]),
            ),
            name: 'completed',
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'completed.dark');
    });
  });

  group('widgets', () {
    testWidgets('continue cta is visible when issuance is completed', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          IssuanceCompleted(isRefreshFlow: false, addedCards: [WalletMockData.card, WalletMockData.altCard]),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.walletPersonalizeSuccessPageContinueCta), findsOneWidget);
    });
  });
}
