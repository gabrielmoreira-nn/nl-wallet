import 'package:flutter/material.dart';

import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/attribute/requested_attribute.dart';
import '../../../domain/model/attribute/ui_attribute.dart';
import '../../../domain/model/card_front.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../domain/model/wallet_card.dart';
import '../../common/widget/animated_linear_progress_indicator.dart';
import '../../common/widget/button/animated_visibility_back_button.dart';
import '../../common/widget/attribute/attribute_row.dart';
import '../../common/widget/bottom_sheet_drag_handle.dart';
import '../../common/widget/card/wallet_card_item.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/confirm_action_sheet.dart';
import '../../common/widget/explanation_sheet.dart';
import '../../common/widget/history/timeline_attribute_row.dart';
import '../../common/widget/history/timeline_card_header.dart';
import '../../common/widget/history/timeline_section_header.dart';
import '../../common/widget/icon_row.dart';
import '../../common/widget/loading_indicator.dart';
import '../../common/widget/pin_header.dart';
import '../../common/widget/policy/policy_row.dart';
import '../../common/widget/policy/policy_section.dart';
import '../../common/widget/select_card_row.dart';
import '../../common/widget/stacked_wallet_cards.dart';
import '../../common/widget/status_icon.dart';
import '../../common/widget/version_text.dart';
import '../../common/widget/wallet_logo.dart';
import '../../verification/model/organization.dart';
import '../theme_screen.dart';

const _kSampleCardFront = CardFront(
  title: 'Sample Card',
  backgroundImage: 'assets/svg/rijks_card_bg_dark.svg',
  theme: CardFrontTheme.dark,
  info: 'Info',
  logoImage: 'assets/non-free/images/logo_card_rijksoverheid.png',
  subtitle: 'Subtitle',
);

class OtherStylesTab extends StatelessWidget {
  const OtherStylesTab({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 32),
      children: [
        _buildSheetSection(context),
        _buildAttributeSection(context),
        _buildCardSection(context),
        _buildHistorySection(context),
        _buildPolicySection(context),
        _buildMiscellaneousSection(context),
      ],
    );
  }

  Widget _buildSheetSection(BuildContext context) {
    return Column(
      children: [
        const ThemeSectionHeader(title: 'Sheets'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'Explanation Sheet'),
        TextButton(
          onPressed: () => {
            ExplanationSheet.show(
              context,
              title: 'Title goes here',
              description: 'Description goes here. This is a demo of the ExplanationSheet!',
              closeButtonText: 'close',
            )
          },
          child: const Text('Explanation Sheet'),
        ),
        const ThemeSectionSubHeader(title: 'Confirm Action Sheet'),
        TextButton(
          onPressed: () => {
            ConfirmActionSheet.show(
              context,
              title: 'Title goes here',
              description: 'Description goes here. This is a demo of the ConfirmActionSheet!',
              cancelButtonText: 'cancel',
              confirmButtonText: 'confirm',
            )
          },
          child: const Text('Confirm Action Sheet'),
        ),
      ],
    );
  }

  Widget _buildAttributeSection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: const [
        ThemeSectionHeader(title: 'Attributes'),
        SizedBox(height: 12),
        ThemeSectionSubHeader(title: 'DataAttributeRow - Type Text'),
        AttributeRow(
          attribute: DataAttribute(
            value: 'This is a DataAttributeRow with type text',
            label: 'Label',
            sourceCardId: 'id',
            valueType: AttributeValueType.text,
            type: AttributeType.other,
          ),
        ),
        ThemeSectionSubHeader(title: 'DataAttributeRow - Type Image'),
        AttributeRow(
          attribute: DataAttribute(
            value: 'assets/non-free/images/image_attribute_placeholder.png',
            label: 'Label: This is a DataAttributeRow with type image',
            sourceCardId: 'id',
            valueType: AttributeValueType.image,
            type: AttributeType.other,
          ),
        ),
        ThemeSectionSubHeader(title: 'RequestedAttributeRow'),
        AttributeRow(
          attribute: RequestedAttribute(
            name: 'This is a RequestedAttributeRow',
            valueType: AttributeValueType.text,
            type: AttributeType.other,
          ),
        ),
        ThemeSectionSubHeader(title: 'UiAttributeRow'),
        AttributeRow(
          attribute: UiAttribute(
            value: 'This is a UiAttributeRow',
            valueType: AttributeValueType.text,
            type: AttributeType.other,
            label: 'Label',
            icon: Icons.remove_red_eye,
          ),
        ),
      ],
    );
  }

  Widget _buildCardSection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: const [
        ThemeSectionHeader(title: 'Cards'),
        SizedBox(height: 12),
        ThemeSectionSubHeader(title: 'WalletCardItem'),
        WalletCardItem(
          title: 'Card Title',
          background: 'assets/svg/rijks_card_bg_dark.svg',
          brightness: Brightness.dark,
          subtitle1: 'Card subtitle1',
          subtitle2: 'Card subtitle2',
          logo: 'assets/non-free/images/logo_card_rijksoverheid.png',
          holograph: 'assets/svg/rijks_holo.svg',
        ),
        ThemeSectionSubHeader(title: 'StackedWalletCards'),
        StackedWalletCards(cards: [_kSampleCardFront, _kSampleCardFront]),
      ],
    );
  }

  Widget _buildHistorySection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const ThemeSectionHeader(title: 'History'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'TimelineCardHeader'),
        const TimelineCardHeader(cardFront: _kSampleCardFront),
        const ThemeSectionSubHeader(title: 'TimelineAttributeRow'),
        TimelineAttributeRow(
          attribute: InteractionTimelineAttribute(
            dateTime: DateTime.now(),
            organization: const Organization(
                id: 'id',
                name: 'Organization Name',
                shortName: 'This is a TimelineAttributeRow',
                description: 'Organization description',
                logoUrl: 'logo'),
            dataAttributes: const [],
            status: InteractionStatus.success,
            policy: const Policy(
              storageDuration: Duration(days: 90),
              dataPurpose: 'Kaart uitgifte',
              dataIsShared: false,
              dataIsSignature: false,
              dataContainsSingleViewProfilePhoto: false,
              deletionCanBeRequested: true,
              privacyPolicyUrl: 'https://www.example.org',
            ),
          ),
          onPressed: () {},
        ),
        const ThemeSectionSubHeader(title: 'TimelineSectionHeader'),
        TimelineSectionHeader(dateTime: DateTime.now()),
      ],
    );
  }

  Widget _buildPolicySection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: const [
        ThemeSectionHeader(title: 'Policy'),
        SizedBox(height: 12),
        ThemeSectionSubHeader(title: 'PolicyRow'),
        PolicyRow(icon: Icons.alarm, title: 'This is a Policy Row'),
        ThemeSectionSubHeader(title: 'PolicySection'),
        PolicySection(
          Policy(
            storageDuration: Duration(days: 90),
            dataPurpose: 'Kaart uitgifte',
            dataIsShared: false,
            dataIsSignature: false,
            dataContainsSingleViewProfilePhoto: false,
            deletionCanBeRequested: true,
            privacyPolicyUrl: 'https://www.example.org',
          ),
        ),
      ],
    );
  }

  Widget _buildMiscellaneousSection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const ThemeSectionHeader(title: 'Miscellaneous'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'AnimatedLinearProgressIndicator'),
        const AnimatedLinearProgressIndicator(progress: 0.3),
        const ThemeSectionSubHeader(title: 'AnimatedVisibilityBackButton'),
        const AnimatedVisibilityBackButton(visible: true),
        const ThemeSectionSubHeader(title: 'BottomSheetDragHandle'),
        const BottomSheetDragHandle(),
        const ThemeSectionSubHeader(title: 'CenteredLoadingIndicator'),
        const CenteredLoadingIndicator(),
        const ThemeSectionSubHeader(title: 'LoadingIndicator'),
        const LoadingIndicator(),
        const ThemeSectionSubHeader(title: 'PinHeader'),
        const PinHeader(title: 'Title', description: 'Description', hasError: false),
        const ThemeSectionSubHeader(title: 'SelectCardRow'),
        SelectCardRow(
          onCardSelectionToggled: (_) {},
          card: const WalletCard(front: _kSampleCardFront, attributes: [], id: 'id'),
          isSelected: true,
        ),
        const ThemeSectionSubHeader(title: 'StatusIcon'),
        const StatusIcon(icon: Icons.ac_unit),
        const ThemeSectionSubHeader(title: 'VersionText'),
        const VersionText(),
        const ThemeSectionSubHeader(title: 'WalletLogo'),
        const WalletLogo(size: 64),
        const ThemeSectionSubHeader(title: 'IconRow'),
        const IconRow(icon: Icon(Icons.ac_unit), text: Text('IconRow'),),
      ],
    );
  }
}