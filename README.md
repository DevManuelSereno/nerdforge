# NerdForge

Hub desktop para RPG/worldbuilding. Tauri v2 + React + TypeScript. Arquitetura file-first (sem banco de dados), com Syncthing embutido para sincronização de vaults.

## Requisitos

- Node.js 20.19+ (recomendado) ou 22.12+
- Rust via rustup
- MSVC Build Tools (Windows)

## Configuração

```bash
npm install
```

### Syncthing

O NerdForge embute o binário do Syncthing. Antes de fazer o build, copie o `syncthing.exe` para:

```
src-tauri/resources/syncthing.exe
```

Você pode baixar o Syncthing para Windows em: https://syncthing.net/downloads/

## Desenvolvimento

```bash
npm run tauri dev
```

## Build

Gera o executável e os instaladores:

```bash
npm run tauri build
```

Após o build, os artefatos ficam em:

- Executável: `src-tauri/target/release/nerdforge.exe`
- Instalador MSI: `src-tauri/target/release/bundle/msi/`
- Instalador NSIS: `src-tauri/target/release/bundle/nsis/`

## Estrutura

- `src/` — frontend React
- `src-tauri/src/` — backend Rust
- `src-tauri/resources/` — binários e recursos embutidos
- `NERDFORGE-CONTEXT.md` — roadmap e contexto do projeto
