import 'dart:async';

import 'package:after_layout/after_layout.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../../util/extension/num_extensions.dart';

/// Widget that fades in (using opacity) the provided [child] based on the scroll offset that
/// it was able to resolve. The scroll offset used for the animation is resolved in based on the
/// following priority:
///   1. Any [ScrollOffset] provided by an ancestor. E.g. using the provided [ScrollOffsetProvider]
///   2. The scroll offset of the provided [scrollController]
///   3. The scroll offset of the [PrimaryScrollController]
/// If none of the above can be resolved, a [UnsupportedError] is thrown.
class FadeInAtOffset extends StatefulWidget {
  /// The offset at which the [child] should start to appear
  final double appearOffset;

  /// The offset at which the [child] should be fully visible
  final double visibleOffset;

  /// The widget that should be fully visible (opacity) at [visibleOffset]
  final Widget child;

  /// The scrollController to observe, if none is provided the widget looks for the PrimaryScrollController.
  final ScrollController? scrollController;

  const FadeInAtOffset({
    this.appearOffset = 0,
    required this.visibleOffset,
    required this.child,
    this.scrollController,
    super.key,
  }) : assert(appearOffset < visibleOffset);

  @override
  State<FadeInAtOffset> createState() => _FadeInAtOffsetState();
}

class _FadeInAtOffsetState extends State<FadeInAtOffset> with AfterLayoutMixin<FadeInAtOffset> {
  bool _afterFirstLayout = false;
  ScrollController? _scrollController;

  @override
  Widget build(BuildContext context) {
    final scrollOffset = context.watch<ScrollOffset?>();

    /// Check if we are ready to build, as before the first layout the _scrollController will not be initialized.
    if (scrollOffset == null && _afterFirstLayout == false) return const SizedBox.shrink();
    final offset = scrollOffset?.offset ?? _scrollController!.offset;
    return Opacity(
      opacity: offset.normalize(widget.appearOffset, widget.visibleOffset).toDouble(),
      child: widget.child,
    );
  }

  @override
  void didUpdateWidget(covariant FadeInAtOffset oldWidget) {
    super.didUpdateWidget(oldWidget);
    _scrollController?.removeListener(_onScroll);
    _scrollController = null;
    if (context.read<ScrollOffset?>() != null) return;
    _scrollController = widget.scrollController ?? PrimaryScrollController.of(context);
    _scrollController?.addListener(_onScroll);
  }

  @override
  FutureOr<void> afterFirstLayout(BuildContext context) {
    if (context.read<ScrollOffset?>() == null) {
      /// No ancestor providing [ScrollOffset], resolve scroll from the scrollController
      _scrollController = widget.scrollController ?? PrimaryScrollController.of(context);
      _scrollController?.addListener(_onScroll);
    }
    _afterFirstLayout = true;
  }

  @override
  void dispose() {
    _scrollController?.removeListener(_onScroll);
    super.dispose();
  }

  void _onScroll() => setState(() {});
}

/// Widget that provides a [ScrollOffset] to it's descendants. By default the [ScrollOffset] is
/// updated based on any incoming [ScrollNotification]s. This behaviour can be overridden with the
/// [observeScrollNotifications] flag.
class ScrollOffsetProvider extends StatelessWidget {
  final Widget child;
  final String debugLabel;
  final bool observeScrollNotifications;

  const ScrollOffsetProvider({
    required this.child,
    this.debugLabel = '',
    this.observeScrollNotifications = true,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider(
      create: (context) => ScrollOffset(debugLabel),
      child: Builder(builder: (context) {
        if (!observeScrollNotifications) return child;
        return NotificationListener<ScrollNotification>(
          onNotification: (notification) {
            context.read<ScrollOffset>().offset = notification.metrics.pixels;
            return false;
          },
          child: child,
        );
      }),
    );
  }
}

/// A simple object to provide a [ScrollController]s offset to other interested widgets that
/// could not otherwise observe it. E.g. useful for sibling widgets that can't rely on the
/// [PrimaryScrollController] or [ScrollNotification]s. In our case it is relevant in the
/// disclose/issue/sign flows, where there is no clear primary [ScrollController] and the
/// [WalletAppBar] can't observe the [ScrollNotification]s because it's a sibling, and not
/// a parent of the scrolling content.
class ScrollOffset extends ChangeNotifier {
  final String debugLabel;

  double _offset = 0.0;

  ScrollOffset(this.debugLabel);

  double get offset => _offset;

  set offset(double value) {
    if (_offset == value) return;
    _offset = value;
    notifyListeners();
  }

  @override
  String toString() => 'ScrollOffset for $debugLabel. Offset: $_offset';
}
