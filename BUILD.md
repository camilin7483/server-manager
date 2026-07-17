# Build

## Requisitos

- **Rust** 1.80+ (edition 2021)
- **Git**
- Dependencias del sistema para eframe:
  - Linux: `libxcb`, `libxkbcommon`, `libwayland`, `libfontconfig`
  - Arch: `sudo pacman -S libxcb libxkbcommon wayland fontconfig`
  - Ubuntu: `sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libfontconfig-dev`

## Compilación

```bash
# Debug
cargo build

# Release optimizado
cargo build --release

# Solo verificar (sin generar binario)
cargo check
```

## Tests

```bash
# Todos los tests
cargo test

# Solo tests de integración
cargo test --test integration

# Crate específico
cargo test -p sm-core
```

## Lint

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

## Perfil de release

Agregar a `Cargo.toml`:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

Compilar release con optimizaciones máximas:

```bash
cargo build --release
```

## CI/CD

Flujo propuesto:

1. `cargo check` — verificar compilación
2. `cargo fmt --check` — formato
3. `cargo clippy -- -D warnings` — lints
4. `cargo test` — tests
5. `cargo build --release` — build final
