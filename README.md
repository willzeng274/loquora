# Loquora Programming Language

~~A functional-inspired programming language designed for AI/LLM-first development, featuring context-sensitive keywords, quaternary operators, and schema-driven development.~~

A concurrent, multi-model programming language designed for synthetic data generation, benchmarking, and evaluation at scale.

Loquora helps orchestrate complex AI conversations, generate high-quality synthetic datasets, and evaluate model performance across multiple providers—all in a type-safe, composable language.

## Planned Features

### Synthetic Data Generation
- Parallel dataset creation
- Multi-model orchestration
- Schema-driven generation
- Conversation simulation

### Evaluation & Benchmarking
- Multi-provider evaluation
- Custom metrics (still designing this)
- Analysis
- A/B testing

### Concurrency & Scale
- Async-first
- Rate limiting
- Streaming support
- Batch processing

## File Extensions (tooling not yet implemented)
- `.loq` - Main source files

## CLI Tools (not yet implemented)
- `loq` (pronounced "lock") - Main CLI and interpreter
- `loqi` (pronounced "low-key") - Package manager
- `loqi.toml` - Package configuration file

## Documentation
- [DESIGN.md](DESIGN.md) - Complete language specification and examples (rough)
- [loquora.ebnf](loquora.ebnf) - Formal grammar definition (this is NOT being read and parsed dynamically, I'm manually implementing the lexer and parser)

## Current Status

The Loquora lexer and parser are currently implemented with a basic interpreter. The language can be tokenized, parsed into an Abstract Syntax Tree (AST), and execute simple expressions. The `cargo run` command provides a REPL for interactive development.

**Not yet implemented:** LLM integration, concurrent execution, synthetic data generation, evaluation frameworks, multi-model orchestration.

## REPL Usage

To start the interactive REPL, simply run:

```bash
cargo run
```

You can then type Loquora code line by line. The REPL supports multiline input; it will prompt with `...>` until a complete statement (ending with a semicolon or a closing brace) is entered. To exit the REPL, type `:q`, `:quit`, `quit`, or `exit` on an empty prompt line.
