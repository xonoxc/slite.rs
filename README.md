# slite.rs

A SQLite implementation written from scratch in Rust for learning purposes.

## Overview

slite.rs is an educational project that aims to build a SQLite-compatible database engine in Rust. The goal is to understand how databases work under the hood by implementing core functionality from scratch.

This project is inspired by the "Build a SQL Database from Scratch" tutorial approach, which breaks down the complex components of a database system into manageable pieces.

## Current Status

This project is in its early stages. Currently implemented:

- Basic CLI with read-eval-print loop (REPL)
- Input buffer handling
- Meta command parsing (e.g., `.exit`)
- Basic statement recognition (`.insert`, `.select`)

## Getting Started

### Prerequisites

- Rust toolchain (cargo)
- Clone the repository

### Build and Run

```bash
cargo build
cargo run
```

### Usage

Once running, you'll see a prompt where you can enter commands:

```sql
> .insert
executing insert!
> .select
executing select
> .exit
```

## Project Structure

```
src/
  main.rs       - Entry point
  cli.rs        - REPL loop and command execution
  cmd.rs        - CLI command parsing
  statements.rs - SQL statement types
  input_buffer.rs - Input handling
```

## Roadmap

- [ ] Implement table structure and row storage
- [ ] Add B-tree index implementation
- [ ] Implement cursor operations
- [ ] Support basic SQL syntax (CREATE, INSERT, SELECT, DELETE)
- [ ] Add query execution engine
- [ ] Implement transaction support
- [ ] Add disk persistence

## Learning Resources

This project follows the incremental approach to learning database internals:

1. REPL and basic parsing
2. In-memory data storage
3. Serialization to disk
4. B-tree indexes for fast lookups
5. Query optimization
6. Transactions and concurrency

## Contributing

This is a learning project. Feel free to experiment and extend the implementation to explore different database concepts.

## License

MIT
