import 'package:material_ui/material_ui.dart';

import 'app_text.dart';
import 'tokens.dart';

/// The one text field of the pages: filters, names, exact values, the
/// command palette. The typed text and the hint are in the monospace font of
/// [TextRole.code] in both looks: FTL's bitmap fonts cannot draw a cursor
/// and a selection, and monospace is the nearest to them
/// (2026-11-19). [search] puts a search icon in front. Its [key] goes to
/// the [TextField] it draws, so a test types into it by that key.
class AppTextField extends StatelessWidget {
  // The key is named `key` for the callers but goes to the drawn field.
  // ignore: use_key_in_widget_constructors
  const AppTextField({
    Key? key,
    this.controller,
    this.hint,
    this.helper,
    this.error,
    this.search = false,
    this.autofocus = false,
    this.keyboardType,
    this.onChanged,
    this.onSubmitted,
  }) : _key = key; // ignore: prefer_initializing_formals

  final Key? _key;

  final TextEditingController? controller;

  /// Shown while the field is empty.
  final String? hint;

  /// A line under the field: the rule or the range of the value.
  final String? helper;

  /// Replaces [helper] in the warning color.
  final String? error;
  final bool search;
  final bool autofocus;
  final TextInputType? keyboardType;
  final ValueChanged<String>? onChanged;
  final ValueChanged<String>? onSubmitted;

  @override
  Widget build(BuildContext context) {
    final tokens = AppTokens.of(context);
    final hint = this.hint;
    return TextField(
      key: _key,
      controller: controller,
      autofocus: autofocus,
      keyboardType: keyboardType,
      style: tokens.style(TextRole.code).copyWith(color: tokens.text),
      decoration: InputDecoration(
        isDense: true,
        border: const OutlineInputBorder(),
        prefixIcon: search ? const Icon(Icons.search) : null,
        hint: hint == null ? null : AppText(hint, role: TextRole.code, color: tokens.muted),
        helperText: helper,
        errorText: error,
      ),
      onChanged: onChanged,
      onSubmitted: onSubmitted,
    );
  }
}
