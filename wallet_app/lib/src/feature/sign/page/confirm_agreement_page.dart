import 'package:flutter/material.dart';

import '../../../data/repository/organization/organization_repository.dart';
import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/widget/app_image.dart';
import '../../common/widget/attribute/data_attribute_row.dart';
import '../../common/widget/button/confirm_buttons.dart';
import '../../common/widget/button/link_button.dart';
import '../../common/widget/policy/policy_section.dart';
import '../../common/widget/sliver_sized_box.dart';

class ConfirmAgreementPage extends StatelessWidget {
  final VoidCallback onDeclinePressed;
  final VoidCallback onAcceptPressed;
  final Policy policy;
  final Organization trustProvider;
  final List<DataAttribute> requestedAttributes;

  const ConfirmAgreementPage({
    required this.onDeclinePressed,
    required this.onAcceptPressed,
    required this.policy,
    required this.trustProvider,
    required this.requestedAttributes,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      child: CustomScrollView(
        slivers: <Widget>[
          const SliverSizedBox(height: 8),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          SliverList(delegate: _getDataAttributesDelegate()),
          SliverToBoxAdapter(child: _buildDataIncorrectButton(context)),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverToBoxAdapter(child: PolicySection(policy)),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverToBoxAdapter(child: _buildTrustProvider(context)),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: Container(
              alignment: Alignment.bottomCenter,
              child: ConfirmButtons(
                onAcceptPressed: onAcceptPressed,
                acceptText: context.l10n.confirmAgreementPageConfirmCta,
                onDeclinePressed: onDeclinePressed,
                declineText: context.l10n.confirmAgreementPageCancelCta,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Image.asset(
            WalletAssets.illustration_sign_2,
            fit: BoxFit.cover,
            width: double.infinity,
          ),
          const SizedBox(height: 32),
          Text(
            context.l10n.confirmAgreementPageTitle,
            style: context.textTheme.displayMedium,
            textAlign: TextAlign.start,
          ),
        ],
      ),
    );
  }

  SliverChildBuilderDelegate _getDataAttributesDelegate() {
    return SliverChildBuilderDelegate(
      (context, index) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: DataAttributeRow(attribute: requestedAttributes[index]),
      ),
      childCount: requestedAttributes.length,
    );
  }

  Widget _buildDataIncorrectButton(BuildContext context) {
    return Align(
      alignment: AlignmentDirectional.centerStart,
      child: LinkButton(
        onPressed: () => PlaceholderScreen.show(context),
        child: Padding(
          padding: const EdgeInsets.only(left: 8),
          child: Text(context.l10n.confirmAgreementPageDataIncorrectCta),
        ),
      ),
    );
  }

  Widget _buildTrustProvider(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Row(
        children: [
          AppImage(asset: trustProvider.logo),
          const SizedBox(width: 16),
          Expanded(
            child: Text(
              context.l10n.confirmAgreementPageSignProvider(trustProvider.displayName.l10nValue(context)),
              style: context.textTheme.bodyLarge,
            ),
          )
        ],
      ),
    );
  }
}
