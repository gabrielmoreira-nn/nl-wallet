part of 'organization_detail_bloc.dart';

sealed class OrganizationDetailState extends Equatable {
  const OrganizationDetailState();
}

class OrganizationDetailInitial extends OrganizationDetailState {
  @override
  List<Object> get props => [];
}

class OrganizationDetailSuccess extends OrganizationDetailState {
  final Organization organization;
  final bool isFirstInteractionWithOrganization;

  const OrganizationDetailSuccess({required this.organization, required this.isFirstInteractionWithOrganization});

  @override
  List<Object> get props => [organization, isFirstInteractionWithOrganization];
}

class OrganizationDetailFailure extends OrganizationDetailState {
  final String organizationId;

  const OrganizationDetailFailure({required this.organizationId});

  @override
  List<Object> get props => [organizationId];
}
