import 'package:flutter/material.dart';

// Simple wrapper [StatefulWidget] that helps to perform a certain
// action once in the initState callback without having to create
// a dedicated StatefulWidget.
class DoOnInit extends StatefulWidget {
  final Function(BuildContext) onInit;
  final Widget child;

  const DoOnInit({
    required this.child,
    required this.onInit,
    Key? key,
  }) : super(key: key);

  @override
  State<DoOnInit> createState() => _DoOnInitState();
}

class _DoOnInitState extends State<DoOnInit> {
  @override
  void initState() {
    super.initState();
    widget.onInit(context);
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
