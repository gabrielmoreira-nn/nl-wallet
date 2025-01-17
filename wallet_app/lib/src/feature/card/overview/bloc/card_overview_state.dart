part of 'card_overview_bloc.dart';

sealed class CardOverviewState extends Equatable {
  const CardOverviewState();
}

class CardOverviewInitial extends CardOverviewState {
  const CardOverviewInitial();

  @override
  List<Object> get props => [];
}

class CardOverviewLoadInProgress extends CardOverviewState {
  const CardOverviewLoadInProgress();

  @override
  List<Object> get props => [];
}

class CardOverviewLoadSuccess extends CardOverviewState {
  final List<WalletCard> cards;

  const CardOverviewLoadSuccess(this.cards);

  @override
  List<Object> get props => [cards];
}

class CardOverviewLoadFailure extends CardOverviewState {
  const CardOverviewLoadFailure();

  @override
  List<Object> get props => [];
}
