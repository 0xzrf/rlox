# Interpreter (Lox)

A small Rust implementation of a **tree-walk interpreter** for the Lox programming language (from *Crafting Interpreters*). This is a personal playground project meant for learning and experimentation.

It is **not** optimized and **not production-ready**. A serious, high-performance interpreter/runtime typically compiles to bytecode and runs on a dedicated VM (often with custom bytecodes, optimizations, and a sandboxed execution environment). This project intentionally stays simpler: it scans, parses, and directly evaluates the AST.

## What’s in the repo

- `crates/scanner`: turns source text into tokens
- `crates/parser`: builds an AST and interprets it
- `crates/types`: shared token/types
- `crates/cli`: a small command-line interface (`interpreter-cli`)
- `test.lox`: a sample program that exercises most supported features

## Supported Lox syntax (currently)

### Expressions

- **literals**: numbers, strings, `true`, `false`, `nil`
- **grouping**: `( ... )`
- **unary**: `!expr`, `-expr`
- **binary arithmetic**: `+`, `-`, `*`, `/`
- **comparisons**: `<`, `<=`, `>`, `>=`
- **equality**: `==`, `!=`
- **logical**: `and`, `or` (short-circuit)
- **variables**: identifier references
- **assignment**: `name = expr`
- **calls**: `callee(arg1, arg2, ...)` (max 255 arguments)

### Statements

- **expression statements**: `expr;`
- **print**: `print expr;`
- **variable declarations**: `var name;` and `var name = expr;`
- **blocks**: `{ ... }` (lexical scoping / shadowing)
- **if / else**:

```lox
if (condition) statement;
if (condition) statement; else statement;
```

- **while**:

```lox
while (condition) statement;
```

- **for** (desugared to `while` by the parser):

```lox
for (initializer; condition; increment) statement;
```

- **functions**:

```lox
fun name(a, b) { 
  return a + b;
}
```

- **return**: `return;` or `return expr;`

### Built-ins

- `clock()` returns a number (seconds since Unix epoch).

## Not implemented (yet)

This interpreter currently focuses on the core “functions + control flow” part of Lox. Notably absent:

- **classes / instances / inheritance** (`class`, `this`, `super`)
- **methods / property access** (`obj.field`)
- **break / continue**

## CLI usage

The CLI currently exposes two subcommands:

- **`tokenize`**: scan a `.lox` file and print tokens
- **`parse`**: scan + parse + interpret a `.lox` file

### Run via Cargo

```bash
cargo run -p cli -- tokenize test.lox
cargo run -p cli -- parse test.lox
```

### Build a binary and run it

```bash
cargo build -p cli
./target/debug/interpreter-cli tokenize test.lox
./target/debug/interpreter-cli parse test.lox
```

### Try the included sample

`test.lox` is intended as a feature smoke-test (variables, expressions, printing, functions, `for`/`while`, closures, recursion, etc.).

## Notes on performance and safety

This is a straightforward AST interpreter and makes no attempt to be fast. If you want speed and stronger isolation, the typical next steps are:

- compile the AST to **bytecode**
- execute on a purpose-built **VM** (custom opcodes, stack/register model)
- add **sandboxing** controls (time limits, memory limits, I/O restrictions)

