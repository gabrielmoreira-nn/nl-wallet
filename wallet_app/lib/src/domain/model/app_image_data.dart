import '../../feature/common/widget/app_image.dart';

/// Represents any image that can be rendered using the [AppImage] widget and aligns
/// with the variants of images that are provided by the wallet_core.
sealed class AppImageData {
  final String data;

  const AppImageData(this.data);
}

class SvgImage extends AppImageData {
  const SvgImage(super.data);
}

class AppAssetImage extends AppImageData {
  const AppAssetImage(super.data);
}

class Base64Image extends AppImageData {
  const Base64Image(super.data);
}
