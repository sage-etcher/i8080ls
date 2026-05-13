# Intel 8080 Assembly Language Server (i8080ls)

Language server for parsing intel 8080 assembly, using CP/M `ASM.COM` inspired
syntax.

Runs over TCP on `127.0.0.1:9292`.

## Features 

- push diagnostics
- initialize
- initialized
- shutdown
- textDocument/didOpen
- textDocument/didClose
- textDocument/didChange
- textDocument/definition
- textDocument/references
- textDocument/hover
- textDocument/prepareRename
- textDocument/rename
- textDocument/semanticTokens/full

## TODO

[ ] support for `SET` macro  
[ ] support for `IF`/`ENDIF` conditionals  
[ ] support for preprocessor arithmatic and logic  
[ ] command line options for stdin/stdout vs tcp operations  
[ ] command line help and version  

