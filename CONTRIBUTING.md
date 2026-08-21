# Contributing to Factory Assembly Language (FAL)

Thank you for your interest in contributing to FAL!
Our goal is to make low-level computer science, assembly, and JIT compiler concepts accessible and intuitive for everyone around the world.

---

## How to Get Involved

1. **Reporting Bugs**: Open an issue describing the unexpected behavior, minimal reproducible `.fal` code, and host OS/CPU information (`fal env`).
2. **Proposing Features**: Start a discussion or feature request issue outlining the proposed syntax, factory mental model mapping, and use case.
3. **Submitting Pull Requests**: Fork the repository, create a descriptive branch, implement changes with tests, and submit a PR.

---

## Development Setup

### Prerequisites
- [Rust & Cargo](https://rustup.rs/) (v1.70+)

### Building from Source
```bash
git clone https://github.com/UnsungRoot/factory-assembly-language.git
cd factory-assembly-language
cargo build
```

### Running Tests
```bash
cargo test
```

### Running Example Programs
```bash
cargo run -- run examples/interactive_calc.fal
cargo run -- run examples/even_odd.fal
cargo run -- run examples/random_guess.fal
cargo run -- env
```

---

## Project Structure

```
FAL/
  src/
    main.rs          - Entry point & module declarations
    cli.rs           - CLI argument parsing (run, env)
    parser.rs        - FAL source code parser
    mapper.rs        - Register & memory mapper
    jit.rs           - x86-64 JIT compiler
    target.rs        - Target platform detection
    falz.rs          - FALZ persistent storage
  examples/          - Example .fal programs
  FAL_Syntax_Sheet.md - Complete syntax reference
```

---

## Coding Standards

- **Zero Unfiltered Unicode / Emojis**: All code, compiler diagnostic output, and documentation should use clean standard ASCII for universal compatibility across minimal terminals.
- **Factory Analogy Integrity**: Every new keyword or construct should clearly map to the real-world factory mental model (Workers, Workbenches, Trays, Storerooms, Supervisors, and Workstations).
- **Safety**: Ensure proper memory bounds, register protection, and error handling in parser, mapper, and JIT modules.
- **Testing**: Every new instruction must have a corresponding unit test in the parser module and be verified via `cargo test`.

---

## License

By contributing to FAL, you agree that your contributions will be licensed under the [MIT License](LICENSE).
