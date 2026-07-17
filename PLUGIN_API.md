# API de Plugins

## Visión general

El sistema de plugins permite extender Server Manager sin modificar el código base. Cada plugin se registra mediante un manifest y puede agregar páginas, widgets, comandos, drivers de conexión y más.

## Manifest

Cada plugin define un archivo `plugin.toml`:

```toml
[plugin]
id = "com.example.my-plugin"
name = "My Plugin"
version = "1.0.0"
description = "Descripción del plugin"
author = "Author Name"
type = "page"
entry_point = "lib.so"

[dependencies]
core = ">=0.1.0"

[permissions]
network = true
filesystem = true
ssh = false
```

## Tipos de plugins

| Tipo | Propósito |
|---|---|
| `page` | Nueva página en el panel central |
| `widget` | Widget para el dashboard |
| `driver` | Protocolo de conexión adicional |
| `protocol` | Soporte para nuevo protocolo |
| `monitor` | Colector de métricas adicional |
| `automation` | Acciones de automatización |
| `theme` | Tema visual |

## Traits del plugin

Los plugins implementan el trait `Plugin` definido en `sm-core`:

```rust
#[async_trait]
pub trait Plugin: Send + Sync {
    fn manifest(&self) -> &PluginManifest;
    async fn init(&mut self, ctx: PluginContext) -> CoreResult<()>;
    async fn shutdown(&mut self) -> CoreResult<()>;
    async fn call(&self, action: &str, args: Value) -> CoreResult<Value>;
}
```

## Contexto del plugin

```rust
pub struct PluginContext {
    pub config: AppConfig,
    pub event_bus: Arc<dyn EventBus>,
    pub db: Arc<dyn Database>,
    pub logger: Arc<dyn LogService>,
}
```

## Registro

```rust
engine.load("/path/to/plugin").await?;
```

## Seguridad

Los plugins operan con permisos granulares:
- `network` — acceso a red
- `filesystem` — lectura/escritura de archivos
- `ssh` — uso de conexiones SSH
- `database` — acceso a la base de datos
- `config` — acceso a la configuración
