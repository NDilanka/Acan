; Each capture is a candidate definition. extract.rs decides module scope and export status.
(function_declaration) @def.function
(generator_function_declaration) @def.function
(class_declaration) @def.class
(abstract_class_declaration) @def.class
(method_definition) @def.method
(interface_declaration) @def.interface
(type_alias_declaration) @def.type
(enum_declaration) @def.enum
(lexical_declaration "const" (variable_declarator) @def.const)
(lexical_declaration "let" (variable_declarator) @def.variable)
(variable_declaration (variable_declarator) @def.variable)
(export_specifier) @def.specifier
(export_statement value: (_)) @def.default
(namespace_export) @def.namespace_export
