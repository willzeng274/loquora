# Loquora Programming Language

A functional-inspired programming language designed for AI/LLM-first development, featuring context-sensitive keywords, quaternary operators, and schema-driven development.

## File Extensions
- `.loq` - Main source files
- `.model.loq` - AI model definitions
- `.schema.loq` - Schema definitions

## CLI Tools
- `loq` (pronounced "lock") - Main CLI
- `loqi` (pronounced "low-key") - Package manager
- `loqi.toml` - Package configuration file

## Documentation
- [DESIGN.md](DESIGN.md) - Complete language specification and examples
- [loquora.ebnf](loquora.ebnf) - Formal grammar definition

## TODO: Missing Language Features

### Template System
- [ ] **Template syntax implementation** - Current EBNF has `template` keyword but unclear semantics
- [ ] **Template parameter interpolation** - How templates integrate with string interpolation
- [ ] **Template composition** - Nested templates, template inheritance
- [ ] **Template validation** - Type checking for template parameters

### Context-Sensitive Keywords
- [ ] **Lexer implementation strategy** - How to handle `break`/`continue`/`return` as both keywords and identifiers
- [ ] **Context detection rules** - Precise rules for when identifiers become keywords
- [ ] **Error handling** - What happens with ambiguous cases like `return return;`

### Standard Library & Built-ins
- [ ] **Core data structures** - `vec()`, `map()`, `set()` function implementations
- [ ] **String manipulation** - Built-in string functions
- [ ] **Mathematical operations** - Math library functions
- [ ] **I/O operations** - File reading, writing, network operations
- [ ] **AI/LLM integration** - Built-in functions for model calls, token counting

### Type System
- [ ] **Type inference** - How much type inference vs explicit annotations
- [ ] **Union types** - `str | int` syntax or alternatives
- [ ] **More Type Definitions** - `type UserId = str;` or similar

### Error Handling
- [ ] **Result/Option types** - Since no try/catch, need success/failure patterns
- [ ] **Error propagation** - How errors bubble up through function calls
- [ ] **Panic semantics** - When and how the language panics

### Module System
- [ ] **Module resolution** - How `import tools.search` resolves to files
- [ ] **Package management** - How `loqi` installs and manages dependencies
- [ ] **Visibility** - Should we have public/private exports

### AI/LLM Features
- [ ] **Model Context Protocol (MCP) integration** - Built-in MCP client/server support
- [ ] **Cost tracking** - Automatic token and cost calculation
- [ ] **Parallel LLM calls** - `task` function for concurrent AI operations
- [ ] **Memory management** - Vector databases, conversation history (P10)
- [ ] **Agent lifecycle** - How agents are created, managed, destroyed (P10)

### Development Tools
- [ ] **Language server** - LSP implementation for IDE support (P5)
- [ ] **Debugger** - Step-through debugging for Loquora code (P6)
- [ ] **Formatter** - Code formatting tools (P9)
- [ ] **Linter** - Static analysis and code quality checks (P4)

### Runtime & Performance
- [ ] **Memory management** - Garbage collection or reference counting (P1)
- [ ] **Concurrency model** - How parallel operations work (P0)
- [ ] **FFI (Foreign Function Interface)** - Should Loquora be able to interface with another language (P0)

### Ecosystem
- [ ] **Standard library** - Core modules and functions (P5)
- [ ] **Package registry** - Central repository for Loquora packages (P5)
- [ ] **Documentation tools** - Auto-generate docs from code (P11)
- [ ] **Testing framework** - Unit testing and integration testing tools (P15)