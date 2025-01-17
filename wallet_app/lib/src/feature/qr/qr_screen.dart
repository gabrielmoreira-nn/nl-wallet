import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../util/extension/build_context_extension.dart';
import '../../wallet_constants.dart';
import '../common/widget/wallet_app_bar.dart';
import 'bloc/flashlight_cubit.dart';
import 'tab/my_qr/my_qr_tab.dart';
import 'tab/qr_scan/bloc/qr_scan_bloc.dart';
import 'tab/qr_scan/qr_scan_tab.dart';
import 'widget/qr_screen_flash_toggle.dart';

class QrScreen extends StatelessWidget {
  const QrScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final tabs = [
      Tab(text: context.l10n.qrScreenScanTabTitle),
      Tab(text: context.l10n.qrScreenMyCodeTabTitle),
    ];
    return BlocProvider(
      create: (context) => FlashlightCubit(),
      child: DefaultTabController(
        length: tabs.length,
        animationDuration: kDefaultAnimationDuration,
        child: Scaffold(
          appBar: WalletAppBar(
            title: Text(context.l10n.qrScreenTitle),
            bottom: TabBar(
              tabs: tabs,
              indicatorPadding: const EdgeInsets.all(1), // Fixes indicator collision with app bar and border (divider)
            ),
            actions: const [QrScreenFlashToggle()],
          ),
          body: TabBarView(
            children: [
              BlocProvider(
                create: (context) => context.read<QrScanBloc?>() ?? QrScanBloc(context.read()),
                child: const QrScanTab(),
              ),
              const MyQrTab(),
            ],
          ),
        ),
      ),
    );
  }
}
