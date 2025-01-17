import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/sheet/confirm_action_sheet.dart';
import '../../common/widget/button/link_button.dart';

/// Builds upon the [ConfirmActionSheet], but supplies defaults for
/// when the user is requesting to stop the disclosure flow.
class DisclosureStopSheet extends StatelessWidget {
  final String organizationName;
  final VoidCallback? onReportIssuePressed;
  final VoidCallback onCancelPressed;
  final VoidCallback onConfirmPressed;

  const DisclosureStopSheet({
    required this.organizationName,
    this.onReportIssuePressed,
    required this.onCancelPressed,
    required this.onConfirmPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return ConfirmActionSheet(
      title: context.l10n.disclosureStopSheetTitle,
      description: context.l10n.disclosureStopSheetDescription(organizationName).addSpaceSuffix,
      cancelButtonText: context.l10n.disclosureStopSheetNegativeCta,
      confirmButtonText: context.l10n.disclosureStopSheetPositiveCta,
      confirmButtonColor: context.colorScheme.error,
      onCancelPressed: onCancelPressed,
      onConfirmPressed: onConfirmPressed,
      confirmIcon: Icons.not_interested,
      extraContent: onReportIssuePressed == null
          ? null
          : LinkButton(
              onPressed: onReportIssuePressed,
              customPadding: const EdgeInsets.all(16),
              child: Text(context.l10n.disclosureStopSheetReportIssueCta),
            ),
    );
  }

  static Future<bool> show(
    BuildContext context, {
    required LocalizedText organizationName,
    VoidCallback? onReportIssuePressed,
  }) async {
    final confirmed = await showModalBottomSheet<bool>(
      context: context,
      isScrollControlled: true,
      builder: (BuildContext context) {
        return Scrollbar(
          child: SingleChildScrollView(
            child: DisclosureStopSheet(
              organizationName: organizationName.l10nValue(context),
              onReportIssuePressed: onReportIssuePressed,
              onConfirmPressed: () => Navigator.pop(context, true),
              onCancelPressed: () => Navigator.pop(context, false),
            ),
          ),
        );
      },
    );
    return confirmed == true;
  }
}
