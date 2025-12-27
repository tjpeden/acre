# AGENTS.md

## Build Commands
- `cargo build` - Build the project
- `cargo run` - Run the application
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run a single test by name
- `cargo clippy` - Run linter
- `cargo fmt` - Format code

## Code Style
- **Edition**: Rust 2024
- **Formatting**: Use `cargo fmt` (rustfmt) before committing
- **Linting**: Code must pass `cargo clippy` with no warnings
- **Imports**: Group std, external crates, then local modules; use `use` statements
- **Naming**: snake_case for functions/variables, PascalCase for types/structs/enums
- **Error Handling**: Use `Result<T, E>` for fallible operations; avoid `.unwrap()` in production code
- **Types**: Prefer explicit types for public APIs; leverage type inference internally

## Framework
- **Bevy 0.17**: ECS game engine - use Systems, Components, Resources patterns
