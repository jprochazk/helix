;;; Indentation for hebi

;; Indent after opening braces and blocks
[
  (block)
  (if_expression)
  (loop_statement)
  (do_expression)
  (list_expression)
  (table_expression)
  (parameter_list)
  (argument_list)
] @indent

;; Outdent closing delimiters
[
  "}"
  "]"
  ")"
] @outdent

;; Indent after 'else' keywords
(if_expression
  "else" @indent
  (#not-same-line? @indent))

;; Don't outdent mid-expression closing delimiters on the same line
(list_expression
  "]" @_end
  (#same-line? @_end)
  (#set! "scope" "all"))

(table_expression
  "}" @_end
  (#same-line? @_end)
  (#set! "scope" "all"))
