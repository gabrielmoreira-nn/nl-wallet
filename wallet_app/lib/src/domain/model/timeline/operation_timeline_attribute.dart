import '../attribute/data_attribute.dart';
import '../localized_text.dart';
import 'timeline_attribute.dart';

class OperationTimelineAttribute extends TimelineAttribute {
  final OperationStatus status;
  final LocalizedText cardTitle;

  const OperationTimelineAttribute({
    required this.status,
    required this.cardTitle,
    required super.dateTime,
    required super.organization,
    required super.dataAttributes,
  }) : super(type: TimelineType.operation);

  @override
  List<Object?> get props => [status, cardTitle, ...super.props];

  @override
  TimelineAttribute copyWith({List<DataAttribute>? dataAttributes, OperationStatus? status}) {
    return OperationTimelineAttribute(
      status: status ?? this.status,
      cardTitle: cardTitle,
      dateTime: dateTime,
      organization: organization,
      dataAttributes: dataAttributes ?? this.dataAttributes,
    );
  }
}

enum OperationStatus { issued, renewed, expired }
