import 'package:core_domain/core_domain.dart';

import '../../../domain/model/pin/pin_validation_error.dart';
import '../mapper.dart';

class PinValidationErrorMapper extends Mapper<PinValidationResult, PinValidationError?> {
  @override
  PinValidationError? map(PinValidationResult input) {
    switch (input) {
      case PinValidationResult.ok:
        return null;
      case PinValidationResult.tooFewUniqueDigitsError:
        return PinValidationError.tooFewUniqueDigits;
      case PinValidationResult.sequentialDigitsError:
        return PinValidationError.sequentialDigits;
      case PinValidationResult.otherError:
        return PinValidationError.other;
    }
  }
}
