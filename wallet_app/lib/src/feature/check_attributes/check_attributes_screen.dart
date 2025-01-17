import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/model/attribute/attribute.dart';
import '../../domain/model/attribute/data_attribute.dart';
import '../../domain/model/wallet_card.dart';
import '../../navigation/secured_page_route.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/formatter/attribute_value_formatter.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/link_tile_button.dart';
import '../common/widget/card/wallet_card_item.dart';
import '../common/widget/sliver_divider.dart';
import '../common/widget/sliver_sized_box.dart';
import '../common/widget/wallet_app_bar.dart';
import 'bloc/check_attributes_bloc.dart';

class CheckAttributesScreen extends StatelessWidget {
  final VoidCallback? onDataIncorrectPressed;

  const CheckAttributesScreen({
    this.onDataIncorrectPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        actions: [
          IconButton(
            onPressed: () => PlaceholderScreen.show(context),
            icon: const Icon(Icons.help_outline_rounded),
          )
        ],
      ),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: _buildContent(context),
            ),
            const BottomBackButton(showDivider: true),
          ],
        ),
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    /// Since the card & attributes don't change throughout the lifecycle of the bloc (see [CheckAttributesBloc])
    /// we can just fetch them directly (without the complexity of a separate BlocBuilder).
    final card = context.read<CheckAttributesBloc>().state.card;
    final attributes = context.read<CheckAttributesBloc>().state.attributes;
    return Scrollbar(
      child: CustomScrollView(
        slivers: [
          SliverToBoxAdapter(
            child: Container(
              padding: const EdgeInsets.all(16),
              alignment: AlignmentDirectional.centerStart,
              child: SizedBox(
                width: 110,
                child: WalletCardItem.fromCardFront(context: context, front: card.front),
              ),
            ),
          ),
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    context.l10n.checkAttributesScreenTitle(
                      attributes.length,
                      attributes.length,
                      card.front.title.l10nValue(context),
                    ),
                    style: context.textTheme.displayMedium,
                    textAlign: TextAlign.start,
                  ),
                  const SizedBox(height: 8),
                  BlocBuilder<CheckAttributesBloc, CheckAttributesState>(builder: (context, state) {
                    switch (state) {
                      case CheckAttributesInitial():
                        return const SizedBox.shrink();
                      case CheckAttributesSuccess():
                        return Text(
                          context.l10n.checkAttributesScreenSubtitle(
                            state.cardIssuer.legalName.l10nValue(context),
                          ),
                          style: context.textTheme.bodySmall,
                          textAlign: TextAlign.start,
                        );
                    }
                  })
                ],
              ),
            ),
          ),
          const SliverSizedBox(height: 24),
          const SliverDivider(height: 1),
          SliverPadding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
            sliver: SliverList.separated(
              itemCount: attributes.length,
              itemBuilder: (context, i) {
                final attribute = attributes[i];
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      attribute.label.l10nValue(context),
                      style: context.textTheme.bodySmall,
                    ),
                    Text(
                      attribute.value.prettyPrint(context),
                      style: context.textTheme.titleMedium,
                    ),
                  ],
                );
              },
              separatorBuilder: (context, i) => const SizedBox(height: 24),
            ),
          ),
          SliverToBoxAdapter(child: _buildDataIncorrectButton(context)),
          const SliverSizedBox(height: 32),
        ],
      ),
    );
  }

  Widget _buildDataIncorrectButton(BuildContext context) {
    if (onDataIncorrectPressed == null) return const SizedBox.shrink();
    return LinkTileButton(
      onPressed: onDataIncorrectPressed,
      child: Text(context.l10n.checkAttributesScreenDataIncorrectCta),
    );
  }

  static void show(
    BuildContext context, {
    required WalletCard card,
    required List<DataAttribute> attributes,
    VoidCallback? onDataIncorrectPressed,
  }) {
    Navigator.push(
      context,
      SecuredPageRoute(
        builder: (c) {
          return BlocProvider<CheckAttributesBloc>(
            create: (context) => CheckAttributesBloc(
              context.read(),
              card: card,
              attributes: attributes,
            )..add(CheckAttributesLoadTriggered()),
            child: CheckAttributesScreen(
              onDataIncorrectPressed: onDataIncorrectPressed,
            ),
          );
        },
      ),
    );
  }
}
