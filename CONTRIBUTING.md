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
git clone https://github.com/your-username/fal.git
cd fal
cargo build
```

### Running Tests
```bash
cargo test
```

### Running Example Programs
```bash
cargo run -- run examples/strings_and_conditions.fal
cargo run -- env
```

---

## Coding Standards

- **Zero Unfiltered Unicode / Emojis**: All code, compiler diagnostic output, and documentation should use clean standard ASCII for universal compatibility across minimal terminals.
- **Factory Analogy Integrity**: Every new keyword or construct should clearly map to the real-world factory mental model (Workers, Workbenches, Trays, Storerooms, Supervisors, and Workstations).
- **Safety**: Ensure proper memory bounds, register protection, and error handling in parser, mapper, and JIT modules.

---

## License

By contributing to FAL, you agree that your contributions will be licensed under the [MIT License](LICENSE).
