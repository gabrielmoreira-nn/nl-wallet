import 'package:fimber/fimber.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'environment.dart';
import 'src/di/wallet_bloc_provider.dart';
import 'src/di/wallet_datasource_provider.dart';
import 'src/di/wallet_repository_provider.dart';
import 'src/di/wallet_service_provider.dart';
import 'src/di/wallet_usecase_provider.dart';
import 'src/feature/lock/auto_lock_observer.dart';
import 'src/wallet_app.dart';
import 'src/wallet_app_bloc_observer.dart';

final GlobalKey<NavigatorState> _navigatorKey = GlobalKey<NavigatorState>();

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Debug specific setup
  if (kDebugMode) {
    Fimber.plantTree(DebugTree());
    Bloc.observer = WalletAppBlocObserver();
  }

  runApp(
    WalletDataSourceProvider(
      child: WalletRepositoryProvider(
        provideMocks: Environment.mockRepositories,
        child: WalletUseCaseProvider(
          child: WalletServiceProvider(
            navigatorKey: _navigatorKey,
            child: WalletBlocProvider(
              child: AutoLockObserver(
                child: WalletApp(
                  navigatorKey: _navigatorKey,
                ),
              ),
            ),
          ),
        ),
      ),
    ),
  );
}