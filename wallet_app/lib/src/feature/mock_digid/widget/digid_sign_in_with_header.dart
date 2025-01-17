import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

class DigidSignInWithHeader extends StatelessWidget {
  const DigidSignInWithHeader({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            context.l10n.mockDigidScreenHeaderTitle,
            style: context.textTheme.displayMedium?.copyWith(color: context.colorScheme.primary),
          ),
          const SizedBox(height: 8),
          Text(
            context.l10n.mockDigidScreenHeaderSubtitle,
            style: context.textTheme.bodyMedium?.copyWith(fontWeight: FontWeight.bold),
          ),
        ],
      ),
    );
  }
}
