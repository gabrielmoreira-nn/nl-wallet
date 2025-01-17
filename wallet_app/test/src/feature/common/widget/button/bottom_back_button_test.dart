import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/widget/button/bottom_back_button.dart';

import '../../../../../wallet_app_test_widget.dart';

void main() {
  group('goldens', () {
    testGoldens('light with divider', (tester) async {
      await tester.pumpWidgetBuilder(
        const BottomBackButton(showDivider: true),
        wrapper: walletAppWrapper(),
        surfaceSize: const Size(200, 300),
      );
      await screenMatchesGolden(tester, 'bottom_back_button/light.divider');
    });

    testGoldens('light no divider', (tester) async {
      await tester.pumpWidgetBuilder(
        const BottomBackButton(showDivider: false),
        wrapper: walletAppWrapper(),
        surfaceSize: const Size(200, 300),
      );
      await screenMatchesGolden(tester, 'bottom_back_button/light');
    });

    testGoldens('dark with divider', (tester) async {
      await tester.pumpWidgetBuilder(
        const BottomBackButton(showDivider: true),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
        surfaceSize: const Size(200, 300),
      );
      await screenMatchesGolden(tester, 'bottom_back_button/dark.divider');
    });

    testGoldens('dark no divider', (tester) async {
      await tester.pumpWidgetBuilder(
        const BottomBackButton(showDivider: false),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
        surfaceSize: const Size(200, 300),
      );
      await screenMatchesGolden(tester, 'bottom_back_button/dark');
    });
  });

  group('widgets', () {
    testWidgets('back button is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const BottomBackButton());

      // Validate that the back button exists
      final buttonFinder = find.text('Back');
      expect(buttonFinder, findsOneWidget);
    });
  });
}
