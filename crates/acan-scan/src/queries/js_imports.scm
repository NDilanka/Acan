; Module specifiers for TypeScript and JavaScript: import, re-export, require(), and import().
(import_statement source: (string) @source)
(export_statement source: (string) @source)
(call_expression function: (identifier) @callee arguments: (arguments . (string) @source) (#eq? @callee "require"))
(call_expression function: (import) arguments: (arguments . (string) @source))
