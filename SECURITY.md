# Seguridad

## Cifrado de credenciales

Las credenciales (contraseñas, claves SSH, tokens) se almacenan cifradas usando **AES-256-GCM**.

### Flujo

1. El usuario provee una **contraseña maestra** al iniciar la aplicación
2. Se genera un **salt aleatorio** de 32 bytes
3. La clave AES-256 se **deriva** mediante SHA-256(password + salt)
4. Cada credencial se serializa a JSON y se cifra con un nonce único
5. El vault completo puede exportarse/importarse cifrado

### Detalles técnicos

- Algoritmo: AES-256-GCM (autenticado)
- Derivación: PKCS#5 PBKDF2-HMAC-SHA256
- Nonce: 96 bits aleatorios por operación
- Zeroization: datos sensibles se borran de memoria tras su uso

## Auditoría

Todas las operaciones sensibles se registran:

- Conexiones y desconexiones
- Comandos ejecutados
- Cambios de configuración
- Autenticaciones (éxito/fallo)
- Operaciones CRUD en servidores

## Almacenamiento

- Base de datos SQLite en `~/.local/share/com.server-manager.server-manager/`
- Sin contraseñas en texto plano en la DB
- Backups cifrados

## Transporte

- SSH via Russh (Rust nativo, sin dependencias C)
- Host key verification
- Soporte para Ed25519, RSA, ECDSA

## Reporte de vulnerabilidades

Por favor reporta vulnerabilidades de seguridad a través de GitHub Issues con la etiqueta `security`.
