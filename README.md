# Loquora Programming Language

A functional-inspired programming language designed for AI/LLM-first development, featuring context-sensitive keywords, quaternary operators, and schema-driven development.

## File Extensions (tooling not yet implemented)
- `.loq` - Main source files
- `.model.loq` - AI model definitions
- `.schema.loq` - Schema definitions

## CLI Tools (not yet implemented)
- `loq` (pronounced "lock") - Main CLI
- `loqi` (pronounced "low-key") - Package manager
- `loqi.toml` - Package configuration file

## Documentation
- [DESIGN.md](DESIGN.md) - Complete language specification and examples (rough)
- [loquora.ebnf](loquora.ebnf) - Formal grammar definition (this is NOT being read and parsed dynamically, I'm manually implementing the lexer and parser)

## Current Status

The Loquora lexer and parser are currently implemented, and right now the language can be tokenized and parsed into an Abstract Syntax Tree (AST). The `cargo run` command provides a REPL that echoes the parsed AST, and it can also read and parse single `.loq` files provided as arguments. Full language functionality, including interpretation and execution, is under active development.

## REPL Usage

To start the interactive REPL, simply run:

```bash
cargo run
```

You can then type Loquora code line by line. The REPL supports multiline input; it will prompt with `...>` until a complete statement (ending with a semicolon or a closing brace) is entered. To exit the REPL, type `:q`, `:quit`, `quit`, or `exit` on an empty prompt line.