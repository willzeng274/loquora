Lexer tokens:
```rs
enum TokenKind {
    // this is not a tagged enum, it has only name
    Int,
    Float,
}

struct Token {
    span: Span, // (basically Range<usize>
    kind: TokenKind,
}

impl Token {
    fn text(input: &str) -> str { index it }
}
```

AST Design:

```rs
struct Spanned<T> {
   inner: T,
   span: Span,
}

enum ExprKind { Int(i64), Let{var: Spanned<String>, value: Expr })

type Expr = Spanned<ExprKind>
```

