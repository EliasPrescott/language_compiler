# Rust Language Compiler

### About
I am trying to build a complete language parser and compiler using Rust. The goal is to not use any external crates or libraries, and to eventually target LLVM or some other well-respected compilation target. I'm not sure what the language's syntax will look like yet, but it will likely be some mix of Rust and F#, as I like the idea of all control structures being expressions.

### Parsing (In-Progress)
I am currently building out the parsing system. It is a hand-made recursive descent parser. I'm trying to keep the parser generic and seperate enough that it could be used for other projects. It can currently only parse brace-defined scopes, basic assignments, and variable references. Most of the basic parsing methods are in-place though, so it should be easier now to add more advanced parsing rules.

#### Current Parsing Example:
Here is some code input that the parser can currently handle:
```
let x = 0;
let y = "hello language";

{
    {
        let testing_var = 23253524;
        testing_var
    }
}

{
    "scoped string"
}
```

Here is the corresponding output:
```Rust
[
    ASTAssignment("x", ASTInteger(0)),
    ASTAssignment("y", ASTString("hello language"))
    ASTScope(
        [
            ASTScope(
                [
                    ASTAssignment("testing_var", ASTInteger(23253524)),
                    ASTVariableRef("testing_var"),
                ],
            ),
        ],
    )
    ASTScope(
        [
            ASTString("scoped string"),
        ],
    )
]
```