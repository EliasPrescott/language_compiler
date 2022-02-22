# Rust Language Compiler

### About
I am trying to build a complete language parser and compiler using Rust. The goal is to not use any external crates or libraries, and to eventually target LLVM or some other well-respected compilation target. I'm not sure what the language's syntax will look like yet, but it will likely be some mix of Rust and F#, as I like the idea of all control structures being expressions.

### Parsing (In-Progress)
I am currently building out the parsing system. It is a hand-made recursive descent parser. I'm trying to keep the parser generic and seperate enough that it could be used for other projects. It can parse integer and string literals, parenthese and brace scopes, initialization and assignments, and functions.

#### Current Parsing Example:
Here is some input that the parser can currently handle:
```
let zero = 0
let x = y = (z = "hello")

let testFunction = (arg) {
    arg
}

let scopeExpression = {
    "scoped string"

    let childEmptyExpression = {

    }
}
```

Here is the corresponding output:
```Rust
[
    ASTInitialization("zero", ASTInteger(0)),
    ASTInitialization(
        "x",
        ASTAssignment(
            "y",
            ASTParentheses(
                [
                    ASTAssignment("z", ASTString("hello"))
                ]
            ),
        ),
    ),
    ASTInitialization(
        "testFunction",
        ASTFunction(
            ASTParentheses(
                [
                    ASTVariableRef("arg")
                ]
            ),
            ASTScope(
                [
                    ASTVariableRef("arg")
                ]
            ),
        ),
    ),
    ASTInitialization(
        "scopeExpression",
        ASTScope(
            [
                ASTString("scoped string"),
                ASTInitialization(
                    "childEmptyExpression",
                    ASTScope(
                        []
                    ),
                ),
            ]
        ),
    ),
]
```