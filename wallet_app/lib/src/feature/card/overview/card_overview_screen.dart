import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_staggered_grid_view/flutter_staggered_grid_view.dart';

import '../../../domain/model/wallet_card.dart';
import '../../../navigation/secured_page_route.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/card/wallet_card_item.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/wallet_app_bar.dart';
import '../detail/argument/card_detail_screen_argument.dart';
import '../detail/card_detail_screen.dart';
import 'bloc/card_overview_bloc.dart';

/// Defines the width required to render a card,
/// used to calculate the crossAxisCount.
const _kCardBreakPointWidth = 300.0;

class CardOverviewScreen extends StatelessWidget {
  const CardOverviewScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('cardOverviewScreen'),
      appBar: _buildAppBar(context),
      body: _buildBody(context),
    );
  }

  PreferredSizeWidget _buildAppBar(BuildContext context) {
    return WalletAppBar(
      title: Text(context.l10n.cardOverviewScreenTitle),
    );
  }

  Widget _buildBody(BuildContext context) {
    return BlocBuilder<CardOverviewBloc, CardOverviewState>(
      builder: (context, state) {
        return switch (state) {
          CardOverviewInitial() => _buildLoading(),
          CardOverviewLoadInProgress() => _buildLoading(),
          CardOverviewLoadSuccess() => _buildCards(context, state.cards),
          CardOverviewLoadFailure() => _buildError(context),
        };
      },
    );
  }

  Widget _buildLoading() {
    return const CenteredLoadingIndicator();
  }

  Widget _buildCards(BuildContext context, List<WalletCard> cards) {
    final crossAxisCount = max(1, (context.mediaQuery.size.width / _kCardBreakPointWidth).floor());
    return MasonryGridView.count(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      crossAxisCount: crossAxisCount,
      mainAxisSpacing: 16,
      crossAxisSpacing: 16,
      itemCount: cards.length,
      itemBuilder: (context, index) => _buildCardListItem(context, cards[index]),
    );
  }

  Widget _buildCardListItem(BuildContext context, WalletCard walletCard) {
    return Hero(
      tag: walletCard.id,
      child: WalletCardItem.fromCardFront(
        context: context,
        front: walletCard.front,
        onPressed: () => _onCardPressed(context, walletCard),
      ),
    );
  }

  void _onCardPressed(BuildContext context, WalletCard walletCard) {
    SecuredPageRoute.overrideDurationOfNextTransition(kPreferredCardDetailEntryTransitionDuration);
    Navigator.restorablePushNamed(
      context,
      WalletRoutes.cardDetailRoute,
      arguments: CardDetailScreenArgument.forCard(walletCard).toJson(),
    );
  }

  Widget _buildError(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          const Spacer(),
          Text(
            context.l10n.errorScreenGenericDescription,
            textAlign: TextAlign.center,
          ),
          const Spacer(),
          ElevatedButton(
            onPressed: () => context.read<CardOverviewBloc>().add(const CardOverviewLoadTriggered()),
            child: Text(context.l10n.generalRetry),
          ),
        ],
      ),
    );
  }
}
