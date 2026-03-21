;;; Text objects for hebi

;; Function definitions
(function_declaration
  body: (block
    "{" @_start "}" @_end
    (#make-range! "function.inside" @_start @_end))) @function.around

(anonymous_function
  body: (block
    "{" @_start "}" @_end
    (#make-range! "function.inside" @_start @_end))) @function.around

;; Parameters
(parameter_list
  "(" @_start ")" @_end
  (#make-range! "parameter.inside" @_start @_end)) @parameter.around

;; Comments
(comment) @comment.inside
(comment) @comment.around

;; Table/list entries
(table_entry) @entry.inside
(table_entry) @entry.around

;; Blocks (general code blocks)
(block
  "{" @_start "}" @_end
  (#make-range! "block.inside" @_start @_end)) @block.around

(if_expression
  consequence: (block
    "{" @_start "}" @_end
    (#make-range! "block.inside" @_start @_end))) @block.around

(if_expression
  alternative: (block
    "{" @_start "}" @_end
    (#make-range! "block.inside" @_start @_end))) @block.around

(loop_statement
  body: (block
    "{" @_start "}" @_end
    (#make-range! "block.inside" @_start @_end))) @block.around

(do_expression
  (block
    "{" @_start "}" @_end
    (#make-range! "block.inside" @_start @_end))) @block.around
