# Contribuir

## Configuración

```bash
git clone https://github.com/cam/server-manager
cd server-manager
cargo build
cargo test
```

## Estilo de código

- Rust 2021 edition
- `cargo fmt` antes de commit
- `cargo clippy` sin warnings
- Comentarios de documentación en las APIs públicas
- Tests para nueva funcionalidad

## Estructura de commits

```
tipo(scope): descripción breve

- feat: nueva funcionalidad
- fix: corrección de bug
- refactor: refactorización
- docs: documentación
- test: tests
- chore: mantenimiento
```

## Flujo de trabajo

1. Fork el repositorio
2. Crea una rama: `git checkout -b feat/mi-feature`
3. Haz tus cambios con tests
4. Ejecuta `cargo fmt && cargo clippy && cargo test`
5. Commit y push
6. Abre un Pull Request

## Reportar bugs

Usa GitHub Issues. Incluye:
- Versión de Server Manager
- SO y arquitectura
- Pasos para reproducir
- Logs relevantes

## Código de conducta

Sé respetuoso. Construimos herramientas para la comunidad.
