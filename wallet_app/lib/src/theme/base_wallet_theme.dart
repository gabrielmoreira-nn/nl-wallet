import 'package:flutter/material.dart';

/// Base Wallet Theme
///
/// Dark / Light classes of the app specify the dedicated colors, but items like textStyles and
/// radii, which are common across the [LightWalletTheme] and [DarkWalletTheme] are specified
/// here as baseThemes, intended to be extended with the correct colors later.

class BaseWalletTheme {
  BaseWalletTheme._();

  //region Font & TextStyles
  static const fontFamily = 'RijksoverheidSansWebText';

  // Only reference through Theme, as fontFamily/color is applied later.
  static const _displayLargeTextStyle = TextStyle(fontSize: 34, fontWeight: FontWeight.bold);
  static const _displayMediumTextStyle = TextStyle(fontSize: 24, fontWeight: FontWeight.bold);
  static const _displaySmallTextStyle = TextStyle(fontSize: 20, fontWeight: FontWeight.bold);
  static const _headlineMediumTextStyle = TextStyle(fontSize: 18, fontWeight: FontWeight.bold);
  static const _headlineSmallTextStyle = TextStyle(fontSize: 24, fontWeight: FontWeight.w400, height: 32 / 24);
  static const _titleMediumTextStyle = TextStyle(fontSize: 16, fontWeight: FontWeight.bold, height: 1.4);
  static const _titleSmallTextStyle = TextStyle(fontSize: 14, fontWeight: FontWeight.bold);
  static const _bodyLargeTextStyle = TextStyle(fontSize: 16, height: 1.5);
  static const _bodyMediumTextStyle = TextStyle(fontSize: 14, height: 1.4);
  static const _labelLargeTextStyle = TextStyle(fontSize: 16, fontWeight: FontWeight.bold);
  static const _bodySmallTextStyle = TextStyle(fontSize: 12);
  static const _labelSmallTextStyle = TextStyle(fontSize: 14, fontWeight: FontWeight.bold);

  static final baseTextTheme = const TextTheme(
    displayLarge: _displayLargeTextStyle,
    displayMedium: _displayMediumTextStyle,
    displaySmall: _displaySmallTextStyle,
    headlineMedium: _headlineMediumTextStyle,
    headlineSmall: _headlineSmallTextStyle,
    titleMedium: _titleMediumTextStyle,
    titleSmall: _titleSmallTextStyle,
    bodyLarge: _bodyLargeTextStyle,
    bodyMedium: _bodyMediumTextStyle,
    labelLarge: _labelLargeTextStyle,
    bodySmall: _bodySmallTextStyle,
    labelSmall: _labelSmallTextStyle,
  ).apply(fontFamily: fontFamily);

  //endregion Font & TextStyles

  //region Button Style & Themes
  static const buttonMinHeight = 48.0;
  static const buttonBorderRadius = 12.0;
  static final buttonShape = RoundedRectangleBorder(borderRadius: BorderRadius.circular(buttonBorderRadius));
  static const buttonTextStyle = TextStyle(fontSize: 16, fontWeight: FontWeight.bold, fontFamily: fontFamily);

  static final baseElevatedButtonTheme = ElevatedButtonThemeData(
    style: ElevatedButton.styleFrom(
      elevation: 0.0,
      textStyle: buttonTextStyle,
      minimumSize: const Size.fromHeight(buttonMinHeight),
      shape: buttonShape,
    ),
  );

  static final floatingActionButtonTheme = FloatingActionButtonThemeData(
    extendedTextStyle: buttonTextStyle,
    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(50)),
  );

  static final outlinedButtonTheme = OutlinedButtonThemeData(
    style: OutlinedButton.styleFrom(
      elevation: 0,
      minimumSize: const Size.fromHeight(buttonMinHeight),
      shape: buttonShape,
      textStyle: buttonTextStyle,
    ),
  );

  static final textButtonTheme = TextButtonThemeData(
    style: TextButton.styleFrom(
      minimumSize: const Size(0.0, buttonMinHeight),
      shape: buttonShape,
    ),
  );

  //endregion Button Themes

  //region Other Themes
  static const baseDividerTheme = DividerThemeData(thickness: 1);

  static const baseBottomSheetTheme = BottomSheetThemeData(
    shape: ContinuousRectangleBorder(),
    showDragHandle: true,
  );

  static const baseBottomNavigationBarThemeData = BottomNavigationBarThemeData(
    elevation: 0.0,
    selectedLabelStyle: TextStyle(fontSize: 12, fontWeight: FontWeight.w700, fontFamily: fontFamily),
    unselectedLabelStyle: TextStyle(fontSize: 12, fontWeight: FontWeight.w400, fontFamily: fontFamily),
  );

  static const baseAppBarTheme = AppBarTheme(centerTitle: true, elevation: 0, scrolledUnderElevation: 0);

  static final tabBarTheme = TabBarTheme(
    labelStyle: baseTextTheme.titleSmall,
    unselectedLabelStyle: baseTextTheme.bodyMedium,
    indicatorSize: TabBarIndicatorSize.tab,
  );

  static const baseScrollbarTheme = ScrollbarThemeData(
    crossAxisMargin: 8.0,
    mainAxisMargin: 8.0,
    radius: Radius.circular(8),
    thickness: MaterialStatePropertyAll(4.0),
    thumbVisibility: MaterialStatePropertyAll(true),
  );

//endregion Other Themes
}
