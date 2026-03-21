;;; Highlighting for hebi

;; Keywords

"import" @keyword.control.import
"as" @keyword.control.import
"from" @keyword.control.import
"if" @keyword.control.conditional
"else" @keyword.control.conditional
"loop" @keyword.control.repeat
"while" @keyword.control.repeat
"for" @keyword.control.repeat
"in" @keyword.control.repeat
"return" @keyword.control.return
"do" @keyword
"let" @keyword
"fn" @keyword.function

[
  (break_expression)
  (continue_expression)
] @keyword.control

;; Import identifiers
(import_statement
  name: (identifier) @variable)

(import_statement
  source: (identifier) @variable)

(import_specifier
  name: (identifier) @variable)

(import_specifier
  alias: (identifier) @variable)

;; Operators

"not" @keyword.operator
"and" @keyword.operator
"or" @keyword.operator

[
  "="
  "+="
  "-="
  "*="
  "/="
  "%="
  "=="
  "!="
  "<="
  ">="
  "<"
  ">"
  "+"
  "-"
  "*"
  "/"
  "%"
  ".."
  "..="
] @operator

;; Punctuation
[","  "." ":"] @punctuation.delimiter

;; Brackets

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

;; Variables
(identifier) @variable

;; Constants
[
  (boolean)
] @constant.builtin.boolean

(nil) @constant.builtin

;; Tables

(table_entry
  key: (field_identifier) @variable.other.member)

(table_entry
  key: (string) @variable.other.member)

;; Functions

(parameter_list
  (identifier) @variable.parameter)

;; For loop variable
(for_statement
  variable: (identifier) @variable.parameter)

(function_declaration
  name: (identifier) @function)

(assignment_expression
  left: (identifier) @function
  right: (anonymous_function))

;; Property/Field access
;; NOTE: Must appear before `call_expression` below
(field_expression
  field: (field_identifier) @variable.other.member)

;; Function/method calls
(call_expression
  function: [
    ((identifier) @function)
    (field_expression
      field: (field_identifier) @function)
  ])

;; Literals
(comment) @comment
(shebang) @comment

(string) @string
(escape_sequence) @constant.character.escape

(integer) @constant.numeric.integer
(float) @constant.numeric.float

;; Errors
(ERROR) @error

