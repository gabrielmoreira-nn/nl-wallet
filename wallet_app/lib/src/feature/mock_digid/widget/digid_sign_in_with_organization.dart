import 'package:flutter/material.dart';

import '../../../domain/model/app_image_data.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/widget/organization/organization_logo.dart';

const _kOrganizationLogoSize = 72.0;

class DigidSignInWithOrganization extends StatelessWidget {
  const DigidSignInWithOrganization({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const OrganizationLogo(
            image: AppAssetImage(WalletAssets.logo_rijksoverheid),
            size: _kOrganizationLogoSize,
          ),
          const SizedBox(height: 16),
          Text(
            context.l10n.mockDigidScreenSignInOrganization,
            style: context.textTheme.displayMedium,
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }
}
