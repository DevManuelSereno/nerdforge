# Guia de Continuação do NerdForge (fora do Hermes)

Este documento contém informações extras necessárias para continuar o desenvolvimento do NerdForge em outro ambiente (ex: Claude Code, OpenCode Go, Codex), que **não estão no `NERDFORGE-CONTEXT.md`**.

Leia este arquivo **junto com** o `NERDFORGE-CONTEXT.md`.

---

## 1. Workflow de desenvolvimento OBRIGATÓRIO

### 1.1 SEMPRE planejar antes de codar

- Nunca comece a implementar código sem aprovação explícita do usuário.
- Apresente o plano da slice/fase, espere ele dizer **"pode começar"**, **"executa"**, **"vai"** ou similar.
- Use linguagem clara e divida em passos pequenos.

### 1.2 Desenvolver em slices verticais

- Uma slice = uma funcionalidade completa de ponta a ponta (backend + frontend + UI).
- Não pule etapas.
- Não comece a próxima slice sem confirmação explícita.

### 1.3 Verificar a cada passo

Depois de qualquer mudança significativa, rode:

```bash
npm run lint
npm run build
```

Para mudanças no backend Rust, rode também:

```bash
cd src-tauri
cargo test
```

Antes de considerar uma slice pronta, rode:

```bash
npm run lint && npm run build
```

E, se alterou código Rust:

```bash
cd src-tauri && cargo test
```

### 1.4 Build de release Tauri

Para builds finais de slice (especialmente se alterou backend ou recursos):

```bash
npm run tauri build
```

Isso gera:
- `src-tauri/target/release/nerdforge.exe`
- `src-tauri/target/release/bundle/msi/NerdForge_*.msi`
- `src-tauri/target/release/bundle/nsis/NerdForge_*-setup.exe`

---

## 2. Ambiente de desenvolvimento

### 2.1 Paths importantes

- **Projeto:** `D:\nerdforge`
- **Home do usuário:** `C:\Users\GAMER`
- **Cargo:** `C:\Users\GAMER\.cargo\bin`
- **MSVC linker:** `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64`

### 2.2 Variáveis de PATH necessárias (MSYS / bash)

Em terminal bash (git-bash / MSYS), exporte antes de rodar Rust:

```bash
export PATH="/c/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools/VC/Tools/MSVC/14.44.35207/bin/Hostx64/x64:$PATH"
export PATH="/c/Users/GAMER/.cargo/bin:$PATH"
```

### 2.3 Toolchain Rust

- **Padrão:** `stable-x86_64-pc-windows-msvc`
- **Versão atual:** `rustc 1.96.0`, `cargo 1.96.0`
- NÃO use a toolchain GNU (`x86_64-pc-windows-gnu`). Ela falha por falta de `dlltool.exe`.

### 2.4 Versões Node

- Node: `v20.18.0`
- npm: `11.6.0`
- Vite prefere Node `^20.19.0 || >=22.12.0`, mas `v20.18.0` funciona com warning.

---

## 3. Scripts de verificação ad-hoc

Sempre que o sistema sinalizar `unverified` ou `stale`, crie um script temporário em `C:\Users\GAMER\AppData\Local\Temp\` com prefixo `hermes-verify-slice-X.Y[-runN].ps1`, execute e remova.

### 3.1 Estrutura mínima do script

```powershell
# hermes-verify-slice-X.Y.ps1
$ErrorActionPreference = "Stop"

Write-Host "=== Slice X.Y - Nome da Slice ===" -ForegroundColor Cyan

# Configura PATH
$rustupHome = "$env:USERPROFILE\.cargo\bin"
$msvcLink = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64"
$env:PATH = "$rustupHome;$msvcLink;$env:PATH"

# Verifica cargo test (se alterou Rust)
$testBat = "$env:TEMP\hermes-verify-cargo-test-X-Y.bat"
"@echo off`ncd /d D:\nerdforge\src-tauri`ncargo test" | Out-File -FilePath $testBat -Encoding ASCII
& cmd /c "$testBat"
$cargoExit = $LASTEXITCODE
Remove-Item $testBat -Force -ErrorAction SilentlyContinue
if ($cargoExit -ne 0) { throw "cargo test failed" }
Write-Host "[PASS] cargo test passed" -ForegroundColor Green

# Verifica arquivos esperados
$requiredFiles = @(
    "D:\nerdforge\src-tauri\src\...",
    "D:\nerdforge\src\pages\...",
    "D:\nerdforge\NERDFORGE-CONTEXT.md"
)
foreach ($file in $requiredFiles) {
    if (-Not (Test-Path $file)) { throw "Missing $file" }
    Write-Host "[PASS] Found $file" -ForegroundColor Green
}

# Verifica symbols/commands no código
$content = Get-Content -Raw -Path "D:\nerdforge\src-tauri\src\..."
if (-Not ($content -match "nome_da_funcao")) { throw "Missing symbol" }

# Verifica NERDFORGE-CONTEXT.md atualizado
$ctx = Get-Content -Raw -Path "D:\nerdforge\NERDFORGE-CONTEXT.md"
if (-Not ($ctx -match "Slice X.Y")) { throw "Context not updated" }

Write-Host "=== Verificacao concluida ===" -ForegroundColor Cyan
```

### 3.2 Regras dos scripts

- Use `.ps1` com `powershell.exe -ExecutionPolicy Bypass -File ...`
- SEMPRE remova o script após execução bem-sucedida.
- Use text anchors ao invés de emoji ✅❌ para verificar o contexto.

---

## 4. Comandos canônicos de verificação

```bash
# Frontend
cd /d/nerdforge
npm run lint
npm run build

# Backend Rust
cd /d/nerdforge/src-tauri
cargo test

# Build completo (demora)
cd /d/nerdforge
npm run tauri build
```

### 4.1 Warnings esperados e inofensivos

O `npm run lint` sempre mostra 2 warnings:

- `button.tsx` exporta `buttonVariants` além do componente
- `WorkspaceContext.tsx` exporta `useWorkspace` além do provider

Não tente "corrigir" esses warnings. Eles são padrão shadcn/ui e React Context.

---

## 5. Convenções do projeto

### 5.1 Arquitetura

- **File-first:** sem banco de dados. O vault em disco é a fonte da verdade.
- **Vault:** `C:\Users\GAMER\Documents\NerdForge\` (ou escolhido pelo usuário no onboarding).
- **Índice do vault:** `.nerdforge/index.json`
- **Config do Syncthing:** `.nerdforge/syncthing-config/`
- **Dados do Syncthing:** `.nerdforge/syncthing-data/`

### 5.2 Backend Rust

- Commands Tauri ficam nos módulos apropriados (`fs.rs`, `config.rs`, `vault.rs`, `indexer.rs`, `parser.rs`, `syncthing/mod.rs`).
- Registre todo novo command em `src-tauri/src/lib.rs`.
- Adicione testes unitários no mesmo arquivo, dentro de `#[cfg(test)] mod tests`.
- Use `serde` para serialização.
- Use `thiserror` ou `Result<T, String>` para erros simples.

### 5.3 Frontend React + TypeScript

- Use path alias `@/` para imports.
- Componentes shadcn/ui ficam em `src/components/ui/`.
- Páginas ficam em `src/pages/`.
- Contextos ficam em `src/contexts/`.
- Helpers em `src/lib/`.
- Use `cn()` de `src/lib/utils.ts` para classes condicionais.

### 5.4 Tailwind + shadcn

- Tema via CSS variables em `src/index.css`.
- Não execute `npx shadcn@latest init` interativo. A CLI é detectada como long-lived e rejeitada.
- Adicione componentes shadcn manualmente quando necessário.

---

## 6. Skills do Hermes usadas neste projeto

Se você tiver acesso a skills em outro ambiente, carregue as equivalentes:

- `nerdforge-desktop-development` — workflow e padrões do projeto NerdForge
- `github-pr-workflow` — se for trabalhar com PRs
- `requesting-code-review` — revisão pré-commit
- `plan` — planejamento em `.hermes/plan.md`
- `test-driven-development` — se quiser TDD

Se não houver skills, use este documento + o `NERDFORGE-CONTEXT.md` como substituto.

---

## 7. Decisões técnicas críticas

### 7.1 Syncthing

- O Syncthing é executado como child process.
- Prioridade de descoberta do executável:
  1. `exe_dir/syncthing.exe` (onde o Tauri coloca no release)
  2. `exe_dir/resources/syncthing.exe`
  3. PATH / Program Files
- A API key é lida do `config.xml` gerado pelo Syncthing.
- O vault é configurado como folder `nerdforge-vault`.

### 7.2 Build

- `syncthing.exe` NÃO é versionado no Git.
- Quem clonar o repo precisa copiar manualmente o `syncthing.exe` para `src-tauri/resources/syncthing.exe`.
- O `src-tauri/target/` também não é versionado.

### 7.3 Testes

- Testes E2E e de componentes React estão adiados até o projeto estar funcionalmente completo.
- Unit tests do backend são obrigatórios para funções novas quando fizer sentido.

---

## 8. Cuidados específicos

### 8.1 Não modificar outro perfil do Hermes

Se estiver no Hermes, não edite skills/plugins/cron/memories de outro perfil sem autorização explícita.

### 8.2 Commits

- O usuário prefere fazer commits manualmente.
- Não faça `git commit` ou `git push` sem permissão explícita.
- Pode inicializar/configurar Git, criar `.gitignore`, README, etc., mas pergunte antes de commitar.

### 8.3 Mensagens de commit

Regras para quando o usuário pedir para gerar uma mensagem de commit:

- Sempre gere um **commit semântico** (Conventional Commits, ex: `feat(escopo): resumo` + corpo), seguindo boas práticas de clareza, legibilidade e entendimento.
- **Não gere** a mensagem de imediato / por conta própria. Espere o usuário pedir explicitamente ("gere a msg de commit").
- **Não faça** nenhum commit automático. Retorne a mensagem na conversa e espere o usuário commitar manualmente.
- **Não inclua** o comando pronto (git-bash). Basta retornar **somente** a mensagem de commit.

### 8.4 Processos em execução

Antes de rodar `npm run tauri build` ou `npm run tauri dev`, mate processos antigos:

```bash
taskkill //IM nerdforge.exe //F 2>/dev/null || true
taskkill //IM syncthing.exe //F 2>/dev/null || true
```

### 8.5 Aviso de Node

O warning do Vite sobre Node `20.19+` é inofensivo. Não quebra o build.

### 8.6 Persistência de workspace

O workspace atual é salvo em memória (React Context). A persistência real (YAML de config) já existe no backend via `config.rs`, mas o frontend pode precisar de refresh ao reabrir.

---

## 9. Checklist antes de dizer "slice pronta"

- [ ] Código implementado
- [ ] Novos commands registrados em `lib.rs` (se houver)
- [ ] `npm run lint` passa (warnings esperados ok)
- [ ] `npm run build` passa
- [ ] `cargo test` passa (se alterou Rust)
- [ ] `npm run tauri build` passa (se alterou backend/recursos)
- [ ] `NERDFORGE-CONTEXT.md` atualizado
- [ ] Verificação ad-hoc executada e removida (se exigido)
- [ ] Usuário informado do resultado
- [ ] Perguntou se pode prosseguir para próxima slice

---

## 10. Comunicação com o usuário

- Responda TODAS as perguntas/observações antes de propor próximos passos.
- Não pule dúvidas para ir direto ao plano.
- Use português brasileiro (o usuário é brasileiro).
- Seja direto, mas completo.
- Sempre que possível, confirme entendimento antes de agir.

---

## 11. Próximos passos conhecidos

Conforme `NERDFORGE-CONTEXT.md`, a próxima fase é:

- **Fase 4 — Dashboard + Projetos**

Não inicie sem aprovação explícita.

---

**Última atualização:** gerado automaticamente durante o desenvolvimento da Fase 3.
