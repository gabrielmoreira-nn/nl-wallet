import 'package:flutter/material.dart';

const kLogoBorderRadius = 4.0;
const kLogoHeight = 40.0;

class CardLogo extends StatelessWidget {
  final String logo;

  const CardLogo({
    required this.logo,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return ClipRRect(
      borderRadius: BorderRadius.circular(kLogoBorderRadius),
      child: Image.asset(logo, height: kLogoHeight),
    );
  }
}
