import 'package:flutter/material.dart';

import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/numbered_list.dart';

class WalletPersonalizeDataIncorrectScreen extends StatelessWidget {
  final VoidCallback onDataRejected;

  const WalletPersonalizeDataIncorrectScreen({required this.onDataRejected, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('personalizeDataIncorrectScreen'),
      appBar: AppBar(
        title: Text(context.l10n.walletPersonalizeDataIncorrectScreenTitle),
      ),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: Scrollbar(
                thumbVisibility: true,
                child: CustomScrollView(
                  slivers: [
                    SliverToBoxAdapter(
                      child: Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
                        child: MergeSemantics(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                context.l10n.walletPersonalizeDataIncorrectScreenSubhead,
                                style: context.textTheme.displayMedium,
                              ),
                              const SizedBox(height: 8),
                              Text(
                                context.l10n.walletPersonalizeDataIncorrectScreenDescription,
                                style: context.textTheme.bodyLarge,
                              ),
                              const SizedBox(height: 24),
                              Text(
                                context.l10n.walletPersonalizeDataIncorrectScreenHowToResolveTitle,
                                style: context.textTheme.bodyLarge?.copyWith(fontWeight: FontWeight.bold),
                              ),
                              NumberedList(
                                items: context.l10n.walletPersonalizeDataIncorrectScreenHowToResolveBulletPoints
                                    .split('\n'),
                              ),
                              const SizedBox(height: 24),
                              Text(
                                context.l10n.walletPersonalizeDataIncorrectScreenNotYourDataTitle,
                                style: context.textTheme.bodyLarge?.copyWith(fontWeight: FontWeight.bold),
                              ),
                              Text(
                                context.l10n.walletPersonalizeDataIncorrectScreenNotYourDataDescription,
                                style: context.textTheme.bodyLarge,
                              ),
                            ],
                          ),
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
            const Divider(height: 1),
            const SizedBox(height: 24),
            _buildBottomSection(context),
          ],
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: ElevatedButton(
            onPressed: onDataRejected,
            child: Text(context.l10n.walletPersonalizeDataIncorrectScreenPrimaryCta),
          ),
        ),
        const BottomBackButton()
      ],
    );
  }

  /// Shows the [WalletPersonalizeDataIncorrectScreen] and also makes sure
  /// the route is popped when the reject button is pressed.
  static void show(BuildContext context, VoidCallback onDataRejected) {
    Navigator.push(
      context,
      SecuredPageRoute(
        builder: (c) => WalletPersonalizeDataIncorrectScreen(
          onDataRejected: () {
            Navigator.pop(c);
            onDataRejected();
          },
        ),
      ),
    );
  }
}
