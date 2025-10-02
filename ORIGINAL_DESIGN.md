# THIS IS NOW OUTDATED, I WILL REWRITE THIS SOON

https://matthijsgroen.github.io/ebnf2railroad/try-yourself.html

# Loquora Programming Language

File extension: `.loq`

CLI: `loq` (pronounced "lock")
Package manager: `loqi` (pronounced "low-key")

Pkg file: `loqi.toml`

# Language Syntax & Structure

TLDR; Functional-inspired syntax with AI/LLM-first design, featuring context-sensitive keywords, quaternary operators, and schema-driven development.

# Keywords
- `template`: prompt templates (parameterized strings)
- `import`/`from`: importing modules or libraries
- `export`: exporting modules or specific items
- `with`: context management
- `schema`: defining schemas for validation and structured outputs
- `struct`: defining structures for data organization (with methods)
- `tool`: defining functions/methods
- `model`: defining AI models with inheritance
- `if`, `else`, `elif`: conditional statements
- `for`, `while`, `loop`: iteration constructs
- `as`: for mapping responses to schemas
- `return`: returning values from functions (context-sensitive)
- `break`, `continue`: loop control (context-sensitive)

Context-sensitive keywords: `return`, `break`, `continue` are only keywords in their respective contexts (tools/loops), otherwise they're valid identifiers.

# Special Operators & Characters
- `//` and `/* */`: comments
- `.`: property access and method calls
- `:`: type annotations
- `->`: return type annotations
- `|`: bitwise OR (no pipe operator in grammar yet)
- `@`: lvalue operator (returns the assignable location)
- `?`: nullable type suffix
- `!`: required/non-optional type suffix
- `?!`: nullable and optional type suffix
- `? :`: ternary conditional operator
- `?? :: !!`: quaternary operator (condition ?? true_val :: false_val !! null_val)

# String & Literal Syntax
- `""`: strings with `{}` interpolation support
- `''`: character literals
- `<<~HEREDOC`: multi-line strings (no interpolation support)
- `.5` or `5.0`: float literals (both forms supported)
- `_private`: identifiers can start with underscore

# Syntax Examples

## Basic Syntax
```loq
// Import syntax - both forms supported
from models.openai import (
	"gpt-4o",
	"gpt-3.5-turbo"
);
import tools.search;
import schema.SearchField;

// Context-sensitive keywords as identifiers (outside their contexts)
break = "this is fine at top level";
return = "also fine here";
continue = "keywords only in their contexts";

// Assignment with different literal types
query = "What is your problem?";
confidence = .95;  // Float starting with decimal
temperature = 0.7;
_private_var = 'x';  // Character literal, underscore identifier

// String interpolation
message = "Query: {query}, confidence: {confidence}";
```

## Schema Definitions
```loq
schema SearchResult {
	title: str;
	url: str;
	some_list: vec<SearchField>;  // Nested schema
	metadata: map<str, str>;
	nullable_field: str?;         // Can be null
	optional_field: str!;         // Required when present
	optional_and_nullable: str?!; // Can be null AND optional
};

schema UserProfile {
	name: str!;
	email: str;
	age: int?;
	preferences: map<str, bool>?!;
};
```

## Struct with Methods
```loq
struct SearchEngine {
	api_key: str;
	base_url: str;
	
	tool search(query: str, limit: int) -> vec<SearchResult> {
		// Nested tool definition allowed
		tool format_query(q: str) -> str {
			return "{q} site:example.com";
		}
		
		formatted = format_query(query);
		return perform_search(formatted, limit);
	}
	
	tool authenticate() {
		if !api_key {
			return false;
		}
		return true;
	}
};
```

## Model Definitions with Inheritance
```loq
model BaseModel {
	temperature = 0.7;
	max_tokens = 1000;
	
	tool generate(prompt: str) -> str {
		return call_api(prompt);
	}
}

model GPT4(BaseModel) {
	model_name = "gpt-4o";
	temperature = 0.2;  // Override base
	
	tool generate(prompt: str) -> str {
		// Override with structured output support
		return enhanced_call(prompt, model_name);
	}
}
```

## Control Flow & Operators
```loq
// Traditional control flow
if user.is_authenticated() {
	welcome_message = "Welcome back!";
} elif user.is_guest() {
	welcome_message = "Welcome, guest!";
} else {
	welcome_message = "Please log in";
}

// Loop constructs with context-sensitive keywords
for (i = 0; i < 10; i = i + 1) {
	if i == 5 {
		continue;  // Valid here
	}
	if i == 8 {
		break;     // Valid here
	}
	process(i);
}

while running {
	result = fetch_data();
	if !result {
		break;  // Context-sensitive keyword
	}
}

loop {
	input = get_input();
	if input == "quit" {
		break;
	}
}

// Ternary operator
status = is_online ? "online" : "offline";

// Quaternary operator (the fun one!)
user_display = user ?? user.name :: "Anonymous" !! panic("No user context!");
result = validate(input) ?? process(input) :: default_value !! error("Invalid state");

// @ operator (lvalue operator)
target @ source * multiplier;  // @ returns lvalue, ignores rvalue
```

## Context Management & Templates
```loq
// Context management
with model("gpt-4o") {
	response1 = ask("What's the weather?");
	response2 = ask("Tell me a joke");
}

// Template definitions
template summarize_text(text: str, max_words: int) {
	"Please summarize the following text in no more than {max_words} words:\n\n{text}\n\nSummary:"
}

// Using templates
summary = ask(summarize_text("Long article text here...", 50));
```

## Advanced Type System
```loq
// Generic-like syntax
storage: map<str, vec<SearchResult>>;
nested_data: vec<map<str, UserProfile?>>;

// Function with complex return types
tool process_results(data: vec<str>) -> map<str, SearchResult?> {
	results = map();
	for (i = 0; i < len(data); i = i + 1) {
		item = get(data, i);
		processed = search(item);
		set(results, item, processed);
	}
	return results;
}
```

# Esoteric & Advanced Examples

## The Quaternary Chaos
```loq
// Nested quaternary operators for maximum confusion
result = condition1 ?? value1 :: backup1 !! panic1 
         ? secondary_check 
         : condition2 ?? value2 :: backup2 !! panic2;

// Quaternary with side effects
user_action = authenticate(user) ?? 
              log_success(user) :: 
              log_failure(user) !! 
              emergency_shutdown("Auth system down");
```

## Context-Sensitive Keyword Abuse
```loq
// These are all valid identifiers at top level!
// Lexer emits IDENTIFIER tokens because no keyword context
break = "not a keyword here";
continue = "also not a keyword";
return = "perfectly fine identifier";

// Assignment context - lexer sees '=' after identifier
result = break;     // 'break' is IDENTIFIER token
value = continue;   // 'continue' is IDENTIFIER token

tool confusing_function() {
	// Inside tool body, 'return' becomes keyword context
	// return = 1;  // ERROR: 'return' is RETURN token here, not identifier
	message = return;     // 'return' is IDENTIFIER here, perfectly valid
	return message;     // 'return' is RETURN keyword token
	// return return; // Not sure how the lexer responds to this yet
}

// Loop contexts make break/continue keywords in statement positions
for (break = 0; break < 10; break = break + 1) {
	// In init/condition/increment: 'break' is identifier (assignment/expression context)
	// In loop body statements: 'break' becomes keyword
	if break == 5 {      // 'break' still identifier in expression
		continue;        // 'continue' is CONTINUE keyword token
	}
	if break == 8 {
		break;           // 'break' is BREAK keyword token
	}
}

while continue < 100 {   // 'continue' as identifier in condition
	continue = continue + 1;  // Assignment to identifier
	if continue > 50 {
		continue;        // 'continue' is CONTINUE keyword token
	}
}

// Lexer implementation determines context:
// - Assignment context (sees '=') → emit IDENTIFIER
// - Statement context in loops → emit BREAK/CONTINUE 
// - Statement context in tools → emit RETURN
// - Expression context → emit IDENTIFIER
```

## @ Operator Shenanigans
```loq
// @ operator returns lvalue, ignores rvalue
weird_assignment @ calculate_complex_value() * 999;
// Equivalent to: weird_assignment = weird_assignment;

// Chain @ operators for ultimate confusion
a @ b @ c * d + e;  // Returns 'a', ignores everything else

// @ in expressions
result = (target @ source.expensive_computation()) + other_value;
```

## Nested Everything
```loq
struct OuterStruct {
	data: str;
	
	tool outer_method() {
		tool inner_helper(x: int) -> str {
			tool deeply_nested() -> bool {
				return true;
			}
			
			if deeply_nested() {
				return "nested success";
			}
			return "nested failure";
		}
		
		return inner_helper(42);
	}
}

model RecursiveModel {
	tool generate_with_model() {
		model InnerModel {
			inner_temp = 0.1;
			
			tool inner_generate() {
				return "deeply nested generation";
			}
		}
		
		inner = InnerModel();
		return inner.inner_generate();
	}
}
```

## Schema Suffix Combinations
```loq
schema EdgeCaseSchema {
	required_field: str!;           // Must be present
	nullable_field: str?;           // Can be null
	optional_nullable: str?!;       // Can be absent OR null
	complex_nested: vec<map<str, UserProfile?>>?!;  // Madness
}
```

## Multiline String with Interpolation
```loq
user_name = "Alice";
score = 95.5;

// Note: multiline strings don't support interpolation
description = <<~REPORT
User Report for Alice
====================

This is a static multiline string without interpolation.
Use regular strings for interpolation instead.
REPORT

// Use regular strings with \n for interpolated multiline content
report = "User Report for {user_name}\n========================\n\nScore: {score}%\nStatus: {score > 90 ? \"Excellent\" : \"Good\"}\n\n{user_name} has {score > 80 ?? \"exceeded\" :: \"met\" !! \"unknown\"} expectations.";
```
# Language Features

## Core Features
- **Prompt Templates**: Define reusable prompt templates with `template` keyword and parameter interpolation.
- **AI Models**: Define models with `model` keyword, supporting inheritance and method overriding.
- **Schemas**: Use `schema` to define data validation schemas with nullable (`?`), optional (`!`), and combined (`?!`) modifiers.
- **Tools/Functions**: Define functions with `tool`, supporting nested definitions, type annotations and return types.
- **Context Management**: Use `with` for context management, similar to Python's `with` statement.
- **Importing**: Use `import` and `from...import` to include external modules or libraries.
- **Structs**: Use `struct` to define data structures with methods, similar to classes in other languages.
- **Control Flow**: Use `if`, `else`, `elif`, `for`, `while`, and `loop` for control flow.

## Advanced Features
- **Context-Sensitive Keywords**: `return`, `break`, `continue` are only keywords in their respective contexts
- **Quaternary Operator**: `condition ?? true_val :: false_val !! null_val` for complex conditional logic
- **@ Operator**: Returns the lvalue (assignable location), ignores rvalue in expressions
- **Nested Declarations**: Tools, models, and other declarations can be nested within each other
- **String Interpolation**: Regular strings (`"text {var}"`) support `{}` interpolation, heredoc strings are static
- **Flexible Float Literals**: Support both `5.0` and `.5` syntax
- **Underscore Identifiers**: Identifiers can start with underscore for private/internal naming

## Functional Design Choices
- **No Array/Object Literals**: Use function calls like `list(1, 2, 3)` or `object(pair("key", value))`
- **No Indexing Syntax**: Use `get(arr, 0)` or `lookup(obj, "key")` for functional access patterns
- **No Variable Keywords**: Assignment creates bindings, immutability handled semantically
- **No Pattern Matching**: Keep grammar simple, use conditional expressions instead
- **No Lambda Functions**: Use named tool declarations for clarity
- **No Try/Catch**: Functions return success/failure values instead of throwing exceptions

## Planned AI/LLM Features
- **Cost & Token Analysis**: Input/output cost built into the models and calculated dynamically
- **Agents**: Defined within structs with built-in lifecycle management
- **Parallelism**: Built-in `task` function for concurrent LLM calls
- **Built-in MCP Support**: Tool integration with Model Context Protocol
- **LLM Judge**: Built-in scoring and evaluation capabilities
- **Retrieval & Memory**: Vector databases and memory management for AI agents
