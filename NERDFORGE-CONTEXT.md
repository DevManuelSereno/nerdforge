# NerdForge — Contexto de Projeto

> **Documento de briefing para sessão de implementação**
> 
> Cole este documento na nova sessão para continuar codando.

---

## Visão Geral

**NerdForge** é um hub desktop para criadores (RPG, worldbuilding, escrita criativa) que unifica:
- Gerenciamento de projetos criativos
- Sincronização entre dispositivos (via Syncthing embutido)
- Geração de conteúdo assistida por IA (via Hermes CLI)
- Integração com ferramentas existentes (Obsidian)

**Filosofia:** File-first, sem banco de dados, vault como sistema de verdade.

---

## Arquitetura Consolidada

```
┌─────────────────────────────────────────────────────────┐
│              NerdForge Desktop (Tauri v2)               │
│                                                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │           React Frontend (WebView)                │  │
│  │  TypeScript + Tailwind + shadcn/ui                │  │
│  └──────────────────┬────────────────────────────────┘  │
│                     │ Tauri IPC (invoke)                │
│  ┌──────────────────▼────────────────────────────────┐  │
│  │              Rust Backend (Core)                   │  │
│  │                                                    │  │
│  │  ┌─────────────┐  ┌────────────┐  ┌───────────┐  │  │
│  │  │  Syncthing  │  │   Vault    │  │  Hermes   │  │  │
│  │  │  Manager    │  │   Engine   │  │  Client   │  │  │
│  │  └─────────────┘  └────────────┘  └───────────┘  │  │
│  │                                                    │  │
│  │  ┌─────────────────────────────────────────────┐  │  │
│  │  │         Module System (Plugin-based)        │  │  │
│  │  │  ┌─────┐ ┌──────┐ ┌──────────┐ ┌────────┐ │  │  │
│  │  │  │ NPC │ │ Lore │ │ Factions │ │ etc... │ │  │  │
│  │  │  └─────┘ └──────┘ └──────────┘ └────────┘ │  │  │
│  │  └─────────────────────────────────────────────┘  │  │
│  ────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐
│  Syncthing.exe  │  │  Vault (files)  │  │  Hermes CLI  │
│   (embedded)    │  │  (markdown/YAML)│  │  (local)     │
└─────────────────┘  └─────────────────┘  └──────────────┘
```

---

## Stack Tecnológica

### Frontend
- **React 18+** (UI framework)
- **TypeScript** (type safety)
- **Tailwind CSS v4** (styling)
- **shadcn/ui** (component library) — **tema padrão, sem customizações fantasy**
- **react-router-dom** (roteamento)
- **Framer Motion** (animações de chat) — *pendente*

### Backend (Rust/Tauri)
- **Tauri v2** (desktop runtime)
- **Tokio** (async runtime)
- **Reqwest** (HTTP client para Syncthing API) — *pendente*
- **Serde + Serde YAML** (parsing de arquivos)
- **Notify** (file watcher)
- **Tracing + Tracing Appender** (logging)
- **Gray Matter** (markdown + frontmatter parser) — *pendente*

### Integrações
- **Syncthing** (sync engine, embutido) — *pendente*
- **Hermes CLI** (IA, via command line) — *pendente*
- **Obsidian** (editor, detectado automaticamente) — *pendente*

---

## Decisões Arquiteturais (O que SIM e o que NÃO fazer)

### ✅ O QUE FAZER

| Decisão | Escolha | Justificativa |
|---------|---------|---------------|
| **Banco de dados** | ❌ Nenhum (file-first) | Vault + Syncthing já resolvem persistência e sync |
| **Armazenamento** | ✅ Arquivos markdown/YAML | Portável, git-friendly, legível |
| **Sync** | ✅ Syncthing embutido | Invisível para o usuário, robusto |
| **IA Backend** | ✅ Hermes CLI | Sem API keys, memória, skills, controle de modelos |
| **IA Modo** | ✅ Híbrido (conversacional + regeneração) | Qualidade na criação, velocidade na regeneração |
| **IA Output** | ✅ Estruturado com delimitadores | `---NPC_START---` e `---NPC_END---` para parsing |
| **Chat UX** | ✅ Sem streaming, com animações | Loading states ricos, typing animation, ações na mensagem |
| **Imagens** | ❌ Não na v1 | Usuário gera por fora, NerdForge só faz prompt |
| **Auth** | ❌ Nenhum (single-user local) | Hobby project, sem multi-tenant |
| **Deploy** | Desktop app (Windows installer) | Tauri gera MSI/NSIS automaticamente |
| **Identidade visual** | ✅ shadcn padrão | Sem customizações fantasy na v1 |

### ❌ O QUE NÃO FAZER (v1)

- ❌ Banco de dados (Supabase, PostgreSQL, etc.)
- ❌ API keys de IA (OpenAI, Anthropic direto)
- ❌ Geração de imagens (DALL-E, Flux, Stable Diffusion)
- ❌ Streaming de chat (palavra por palavra)
- ❌ Multi-tenant / auth de usuários
- ❌ Tema fantasy customizado
- ❌ Módulos complexos (voz, tokens VTT, handouts automáticos)

---

## Ambiente de Desenvolvimento Atual

- **OS**: Windows 10
- **Projeto**: `D:\nerdforge`
- **Node.js**: v20.18.0 (warning do Vite, funciona mas prefere 20.19+)
- **npm**: 11.6.0
- **Rust**: 1.96.0 (toolchain `stable-x86_64-pc-windows-msvc`)
- **Visual Studio BuildTools 2022** com carga C++
- **PATH manual necessário por sessão**:
  - `C:\Users\GAMER\.cargo\bin`
  - `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64`

---

## Modelo de Dados (File-First)

### Estrutura do Vault

```
📁 World End Tales/
├── 📁 NPCs/
│   └── 📁 Kael Thornwood/
│       ├── index.md              ← lore + metadata (frontmatter)
│       ├── visual-dna.yaml       ← aparência estruturada
│       ├── prompt.md             ← visual prompt final
│       ├── portrait.png          ← imagem gerada (opcional)
│       └── history/              ← versões anteriores
│           ├── v1.md
│           └── v2.md
├── 📁 Factions/
├── 📁 Locations/
├── 📁 Lore/
└── 📁 .nerdforge/                ← metadata do sistema
    ├── config.yaml               ← configuração do workspace
    ├── workspace.yaml            ← dispositivos, projetos
    ├── cache/
    │   └── hermes-responses.json ← cache de chamadas
    └── index.json                ← metadata cache (performance)
```

### Pastas Criadas Automaticamente pelo Vault Engine

Atualmente, `initialize_vault()` cria:
- `npcs`
- `lore`
- `factions`
- `locations`
- `items`
- `scenes`
- `timeline`
- `.nerdforge`
- `.nerdforge/index`
- `.nerdforge/cache`

### Schema de Arquivos

**`NPCs/Kael Thornwood/index.md`**
```markdown
---
id: kael-thornwood
created: 2026-06-27
updated: 2026-06-27
module: npc
status: complete
portrait: ./portrait.png
visual_dna: ./visual-dna.yaml
tags: [npc, faction-thornwood, act-2]
---

# Kael Thornwood

## Conceito
Um mercador de informações que opera nas sombras de Valdris.

## Lore
Kael nasceu nas docas de Valdris, filho de...

## Personalidade
Calculista, mas com um código de honra próprio...

## Papel Narrativo
Informante relutante do grupo na Act 2...
```

**`NPCs/Kael Thornwood/visual-dna.yaml`**
```yaml
version: 1
last_updated: 2026-06-27

face:
  shape: angular
  features:
    - scar across left eyebrow
    - stubble, never clean-shaven

skin:
  tone: olive, weathered
  marks:
    - tattoo of compass on right wrist

eyes:
  color: dark brown
  shape: narrow, calculating
  distinctive: null

hair:
  color: black with early gray at temples
  style: slicked back, loose strands
  length: medium

body:
  build: lean, wiry
  height: average
  age_appearance: late 30s

clothing:
  style: urban rogue, merchant-class
  colors: [dark green, charcoal, brass accents]
  distinctive_items:
    - leather satchel with brass clasps
    - signet ring, family crest worn smooth

art_style:
  medium: digital painting
  mood: dark fantasy, noir-tinged
  reference: Craig Mullins
  aspect_ratio: "3:4"
  orientation: portrait
```

**`.nerdforge/config.yaml`**
```yaml
workspace:
  name: Meu Vault
  root_path: C:\Users\GAMER\Documents\NerdForge

modules:
  npc:
    enabled: true
    default_style: dark-fantasy
  lore:
    enabled: true

sync:
  engine: syncthing
  embedded: true
  api_key: auto-generated

hermes:
  mode: cli
  context_window: 20
  auto_summarize: true
```

---

## Roadmap Completo (6 Fases, ~31 Slices)

Legenda:
- ✅ Concluído
- 🔄 Em andamento / próximo
- ⏳ Pendente

---

### **FASE 1 — Fundação Tauri** ✅

#### ✅ Slice 1.1 — Setup do Projeto
- Criar projeto Tauri v2 com React + TypeScript
- Configurar Tailwind CSS
- Instalar shadcn/ui (tema padrão)
- Estrutura de pastas básica
- Configurar ESLint + Prettier

**Critérios de pronto:**
- ✅ `npm run dev` abre a aplicação
- ✅ Window aparece com título "NerdForge"
- ✅ Tailwind funcionando
- ✅ shadcn/ui instalado

---

#### ✅ Slice 1.2 — Window Management
- Configurar single instance (não abre 2x)
- System tray (ícone na barra de tarefas)
- Menu de contexto do tray (Abrir, Sair)
- Fechar para tray (não fecha app)

**Critérios de pronto:**
- ✅ Só uma instância do app pode rodar
- ✅ Ícone aparece no system tray
- ✅ Clicar direito no tray mostra menu
- ✅ Fechar window minimiza para tray

---

#### ✅ Slice 1.3 — Filesystem Access
- Tauri commands: `read_file`, `write_file`, `list_dir`, `create_dir`
- File watcher (detecta mudanças em tempo real)
- Permissões do Tauri (fs:read, fs:write)

**Critérios de pronto:**
- ✅ Posso ler arquivo do disco via Tauri command
- ✅ Posso escrever arquivo
- ✅ Posso listar diretório
- ✅ File watcher detecta mudança e emite evento

---

#### ✅ Slice 1.4 — Config Persistence
- Criar `.nerdforge/config.yaml` automaticamente
- Ler/escrever config via Tauri commands
- Schema de config (workspace name, modules enabled, etc.)

**Critérios de pronto:**
- ✅ Config é criada na primeira execução
- ✅ Posso ler config via command
- ✅ Posso atualizar config e persistir
- ✅ Validação de schema

---

#### ✅ Slice 1.5 — Logging & Error Handling
- Logging para arquivo em `%APPDATA%/NerdForge/logs/`
- Níveis de log: INFO, WARN, ERROR, DEBUG
- Erros padronizados nos commands
- Frontend exibe erros de forma amigável

**Critérios de pronto:**
- ✅ Logs escritos em arquivo
- ✅ Níveis de log configurados
- ✅ Erros padronizados
- ✅ Logs em operações principais

---

#### ✅ Slice 1.6 — Frontend Shell
- Layout principal (sidebar + header + content area)
- Navegação entre seções (Dashboard, NPC, Lore, Vault, Settings)
- Roteamento com react-router-dom
- Componentes base (Button, Card, Input)

**Critérios de pronto:**
- ✅ Sidebar com navegação funcional
- ✅ Posso trocar entre seções
- ✅ Layout responsivo com drawer mobile
- ✅ Componentes shadcn renderizam corretamente

---

#### ✅ Slice 1.7 — Onboarding Flow
- Detectar first-run (`is_first_run`)
- Tela de boas-vindas
- Criar workspace (nome + caminho)
- Salvar config e redirecionar para dashboard

**Critérios de pronto:**
- ✅ Detecta first-run
- ✅ Tela de onboarding aparece
- ✅ Cria config ao confirmar
- ✅ Redireciona para app principal

**Observação:** O botão de ícone de pasta no onboarding **ainda não abre seletor de pasta**. Ele apenas restaura o caminho padrão. O file picker nativo será implementado em slice futuro.

---

### **FASE 2 — Vault Engine** 🔄

#### ✅ Slice 2.1 — Vault Folder Structure
- Estrutura de pastas padrão (`npcs/`, `lore/`, `factions/`, `locations/`, `.nerdforge/`)
- Comando Tauri `initialize_vault(root)`
- Comando Tauri `get_vault_structure(root)`
- Integração com onboarding (cria pastas ao criar workspace)
- Página Vault exibe estrutura

**Critérios de pronto:**
- ✅ Pastas padrão são criadas automaticamente
- ✅ Posso listar estrutura do vault
- ✅ Onboarding inicializa vault
- ✅ Página Vault exibe status das pastas

---

#### ✅ Slice 2.2 — Vault Indexer
- Scan recursivo de arquivos markdown/YAML
- Extrair metadata básica (caminho, módulo, tamanho, data de modificação)
- Construir e salvar `.nerdforge/index.json`
- Commands: `rebuild_index`, `get_index`
- Página Vault exibe índice agrupado por módulo

**Critérios de pronto:**
- ✅ Index é criado automaticamente
- ✅ Reflete estado atual do vault
- ✅ Posso reconstruir índice manualmente
- ✅ UI mostra arquivos por módulo

---

#### ✅ Slice 2.3 — File Parser
- Markdown + frontmatter parser
- YAML parser (serde_yaml)
- Schema validation básica para `visual-dna.yaml`
- Error handling para arquivos corrompidos
- Integração com indexer (enriquece entradas com title/status/tags)
- Commands: `parse_markdown_file`, `parse_yaml_file`, `validate_visual_dna`

**Critérios de pronto:**
- ✅ Posso ler markdown com frontmatter
- ✅ Posso ler YAML
- ✅ Validação de schema funciona
- ✅ Erros são tratados
- ✅ Index enriquecido com metadados do frontmatter

---

### **FASE 3 — Syncthing Embutido** ⏳

#### ✅ Slice 3.1 — Process Manager
- Detectar binário do Syncthing (`syncthing.exe`)
- Tauri command: `start_syncthing()`
- Spawn como child process
- Commands: `start_syncthing`, `stop_syncthing`, `syncthing_status`
- UI básica na página Settings

**Critérios de pronto:**
- ✅ Syncthing inicia via command
- ✅ Syncthing para via command
- ✅ Status é mostrado na UI
- ⏳ Bundle `syncthing.exe` em `resources/` (futuro)
- ⏳ Health check confirma que está rodando (Slice 3.2)
- ⏳ Syncthing fecha quando NerdForge fecha (Slice 3.2)
- ⏳ Se crashar, reinicia automaticamente (Slice 3.2)

---

#### ✅ Slice 3.2 — Syncthing API Client
- HTTP client (reqwest) para `localhost:8384`
- Endpoints: `/rest/system/ping`, `/rest/system/status`, `/rest/config/folders`
- Auth via API key lida do `config.xml`
- Health check com latência
- Commands: `syncthing_health_check`, `syncthing_system_status`, `syncthing_folders`
- UI mostra versão, ID, uptime e folders

**Critérios de pronto:**
- ✅ Posso chamar API do Syncthing via Rust
- ✅ API key é lida do config.xml
- ✅ Health check funciona
- ✅ Erros são tratados

---

#### ✅ Slice 3.3 — Syncthing Onboarding/Settings
- Configurar folder de sync após criar workspace
- Mostrar status de sync no dashboard
- Tela de dispositivos (placeholder inicial)
- Commands: `configure_vault_folder`, `get_vault_folder_status`
- Botão "Sincronizar este vault" na Settings
- Vault registrado como folder `sendreceive`

**Critérios de pronto:**
- ✅ Syncthing inicia após onboarding
- ✅ Vault é configurado como folder no Syncthing
- ✅ Status de sync visível no dashboard

---

#### ✅ Slice 3.4 — Syncthing Bundle
- Copiar `syncthing.exe` para `src-tauri/resources/`
- Configurar `tauri.conf.json` para embutir resource
- Process manager prioriza executável bundled
- Fallback para Syncthing do sistema (PATH/pastas comuns)
- Instalador inclui Syncthing

**Critérios de pronto:**
- ✅ `syncthing.exe` está na pasta resources
- ✅ Build inclui o executável no bundle
- ✅ NerdForge encontra e usa o bundled syncthing

---

#### ✅ Slice 3.5 — Device Pairing
- Mostrar Device ID local (do Syncthing)
- Botão para copiar Device ID
- Campo para colar Device ID de outro dispositivo
- Botão "Adicionar dispositivo" via API do Syncthing
- Listar dispositivos conhecidos/pendentes
- Aceitar pedidos de conexão recebidos
- Tudo dentro do app NerdForge, sem abrir navegador

**Critérios de pronto:**
- ✅ Device ID visível e copiável
- ✅ É possível adicionar dispositivo por ID
- ✅ Lista de dispositivos sincronizados visível
- ✅ Pedidos pendentes podem ser aceitos

---

#### ✅ Slice 3.6 — Share Vault with Device
- Compartilhar folder `nerdforge-vault` com dispositivos emparelhados
- Mostrar quais dispositivos têm acesso ao vault
- Compartilhamento automático ao adicionar novo dispositivo
- Botão "Compartilhar vault" para dispositivos existentes

**Critérios de pronto:**
- ✅ Vault compartilhado automaticamente ao adicionar dispositivo
- ✅ UI mostra dispositivos com acesso ao vault
- ✅ É possível compartilhar vault com dispositivo já emparelhado

---

### **FASE 4 — Dashboard + Projetos** ⏳

#### ⏳ Slice 4.1 — Dashboard
- Status geral (Syncthing, dispositivos, projetos)
- Timeline de atividades recentes
- Quick actions (novo projeto, adicionar dispositivo)

**Critérios de pronto:**
- ⏳ Dashboard mostra status do Syncthing
- ⏳ Lista dispositivos conectados
- ⏳ Lista projetos
- ⏳ Timeline de atividades

---

#### ⏳ Slice 4.2 — Lista de Projetos
- Cards com nome, status, última modificação
- Abrir projeto (carrega vault)
- Criar novo projeto (wizard)
- Deletar projeto (confirmação)

**Critérios de pronto:**
- ⏳ Lista projetos como cards
- ⏳ Posso abrir projeto
- ⏳ Posso criar projeto
- ⏳ Posso deletar projeto

---

#### ⏳ Slice 4.3 — Criar Projeto
- Nome do projeto
- Pasta (file dialog)
- Template (RPG, Livro, Personalizado)
- Cria `.nerdforge/config.yaml` automaticamente

**Critérios de pronto:**
- ⏳ Wizard de criação funcional
- ⏳ Cria estrutura de pastas
- ⏳ Cria config.yaml
- ⏳ Projeto aparece na lista

---

### **FASE 5 — Módulos** ⏳

#### ⏳ Slice 5.1 — Hermes CLI Integration
- Tauri command: `chat_with_hermes(prompt, history)`
- Spawn `hermes` CLI como child process
- Captura stdout (resposta completa)
- Timeout handling (30s)
- Error handling (Hermes não encontrado)

**Critérios de pronto:**
- ⏳ Posso chamar Hermes via CLI
- ⏳ Resposta é capturada
- ⏳ Timeout funciona
- ⏳ Erros são tratados

---

#### ⏳ Slice 5.2 — Contexto de Conversa
- Estrutura de dados: `Conversation { messages, npc_context }`
- Monta prompt com histórico (últimas 20 mensagens)
- Auto-summarize (a cada 20 mensagens, gera resumo)
- Persiste conversa localmente (`.nerdforge/conversations/`)

**Critérios de pronto:**
- ⏳ Histórico é mantido
- ⏳ Prompt inclui contexto
- ⏳ Auto-summarize funciona
- ⏳ Conversa é persistida

---

#### ⏳ Slice 5.3 — UI de Chat
- Componente React: `ChatInterface`
- Bolhas de mensagem (user + hermes)
- Loading state rico (animação, texto rotativo, progress bar fake)
- Animação de "digitação" quando resposta chega (1-2s)
- Histórico de conversa (scroll, busca)

**Critérios de pronto:**
- ⏳ Chat UI funcional
- ⏳ Loading state agradável
- ⏳ Animação de digitação
- ⏳ Histórico scrollável

---

#### ⏳ Slice 5.4 — Structured Output
- Skill: "gerar NPC estruturado" (Hermes gera YAML com delimitadores)
- Parser de resposta (extrair entre `---NPC_START---` e `---NPC_END---`)
- YAML parser (serde_yaml)
- Validação de schema
- Error handling (YAML inválido)

**Critérios de pronto:**
- ⏳ Hermes gera output estruturado
- ⏳ Parser extrai YAML
- ⏳ Validação funciona
- ⏳ Erros são tratados

---

#### ⏳ Slice 5.5 — Módulo NPC
- Botão "Novo NPC" abre chat
- Hermes faz perguntas (concept, contexto, função narrativa)
- Usuário responde, Hermes refina
- Após 3-5 mensagens, Hermes gera output estruturado
- Cards editáveis (lore, Visual DNA, Visual Prompt)
- Botão "Aprovar e Salvar"
- Salva em `npcs/<nome-do-npc>/`

**Critérios de pronto:**
- ⏳ Fluxo conversacional funciona
- ⏳ Cards editáveis
- ⏳ Aprovação salva nos arquivos
- ⏳ NPC aparece na lista

---

#### ⏳ Slice 5.6 — Regeneração Rápida
- Botões: "Regenerar Lore", "Regenerar Visual DNA", "Regenerar Visual Prompt"
- Cada chamada é independente (sem conversa)
- Histórico de versões (salva versão anterior em `history/v1.md`)
- Export de Visual Prompt (copiar para clipboard)
- Botão "Adicionar Imagem" (upload, salva em `portrait.png`)

**Critérios de pronto:**
- ⏳ Regeneração funciona
- ⏳ Histórico de versões
- ⏳ Export de prompt
- ⏳ Upload de imagem

---

#### ⏳ Slice 5.7 — Módulo Lore
- Reusa 80% da infra do NPC
- Estrutura: `lore/<categoria>/<titulo>/index.md`
- Chat para criar artigos de lore
- Tags e cross-references

**Critérios de pronto:**
- ⏳ Criar lore via chat
- ⏳ Salva na estrutura correta
- ⏳ Lista de artigos de lore

---

#### ⏳ Slice 5.8 — Módulos Factions e Locations
- Similar ao Lore
- Factions: `factions/<nome>/index.md`
- Locations: `locations/<nome>/index.md`
- Cross-referencing com NPCs

**Critérios de pronto:**
- ⏳ Factions criáveis
- ⏳ Locations criáveis
- ⏳ Referências entre entidades

---

### **FASE 6 — Polish + Conflitos** ⏳

#### ⏳ Slice 6.1 — Integração com Obsidian
- Botão "Abrir no Obsidian" (abre `index.md` no vault)
- Detecta Obsidian automaticamente (process/port check)
- URI scheme: `obsidian://open?path=<vault>&file=<npc>`

**Critérios de pronto:**
- ⏳ Detecta Obsidian
- ⏳ Botão abre NPC no Obsidian
- ⏳ URI scheme funciona

---

#### ⏳ Slice 6.2 — Detecção de Conflitos
- Monitora arquivos `.sync-conflict-*` (Syncthing)
- Lista conflitos na UI
- Notificação visual (badge, toast)

**Critérios de pronto:**
- ⏳ Detecta conflitos
- ⏳ Lista na UI
- ⏳ Notificação aparece

---

#### ⏳ Slice 6.3 — UI de Resolução de Conflitos
- Side-by-side diff (versão local vs remota)
- Botões: "Manter Local", "Manter Remota", "Mesclar"
- Preview de mudanças antes de aplicar

**Critérios de pronto:**
- ⏳ Diff side-by-side
- ⏳ Botões funcionais
- ⏳ Preview de mudanças

---

#### ⏳ Slice 6.4 — Health Check
- Status de Syncthing (online/offline)
- Status de Hermes (instalado, versão)
- Status de Obsidian (detectado, vaults encontrados)
- Dashboard mostra health geral

**Critérios de pronto:**
- ⏳ Health check de todas integrações
- ⏳ Dashboard mostra status
- ⏳ Alertas de problema

---

#### ⏳ Slice 6.5 — Auto-Update do Syncthing
- Verifica versão mais recente (GitHub releases)
- Download automático (background)
- Substitui `syncthing.exe` (com backup)
- Reinicia processo

**Critérios de pronto:**
- ⏳ Verifica updates
- ⏳ Download automático
- ⏳ Substitui exe
- ⏳ Reinicia processo

---

#### ⏳ Slice 6.6 — Installer Windows
- Tauri gera MSI/NSIS automaticamente
- Icon, metadata, version
- Start with Windows (registry key opcional)

**Critérios de pronto:**
- ⏳ Installer gerado
- ⏳ Instala corretamente
- ⏳ Start with Windows funciona

---

## Prioridade de Módulos

1. **NPC** (primeiro) — valida toda a arquitetura
2. **Lore** (segundo) — reusa 80% da infra do NPC
3. **Factions** (terceiro) — introduz cross-referencing
4. **Locations** (quarto) — similar a Factions
5. **Cross-Referencing** (feature transversal) — precisa dos módulos acima

---

## Modelo Recomendado

**Modelo principal:** `Kimi K2.7 Code Med`

**Alternativas:**
- Problemas complexos de Rust: `Deepseek V4 Pro Med`
- Planejamento/arquitetura: `Qwen3.7 Max Med`

**Esforço:**
- Fase 1-2: **Med**
- Fase 3 (Syncthing): **Med**
- Fase 4 (Dashboard): **Med**
- Fase 5 (Modules): **High**
- Fase 6 (Polish): **Med**
- Debugging difícil: **Max**
- Boilerplate simples: **Low**

---

## Fluxo de Trabalho

```
1. Eu implemento (escrevo código, crio arquivos, faço mudanças)
   ↓
2. Você revisa (lê o código, testa, valida)
   ↓
3. Você aprova OU pede mudanças
   ↓
4. Se pediu mudanças → eu ajusto → volta para passo 2
   ↓
5. Se aprovou → marcamos vertical slice como completo
   ↓
6. Próximo vertical slice
```

### Regras

- **Eu não começo a codar sem sua aprovação explícita** ("pode começar", "vai", "executa")
- **Você revisa tudo** antes de avançar para o próximo slice
- **Dúvidas são resolvidas no planejamento**, não durante implementação
- **Cada vertical slice tem critérios de "pronto"** definidos antes de começar

---

## Resumo de Progresso

| Fase | Slices | Status | Tempo Estimado |
|------|--------|--------|----------------|
| Fase 1 — Fundação | 7 slices | ✅ Concluída | ~12 horas |
| Fase 2 — Vault Engine | 3 slices | 🔄 Em andamento | ~10 horas |
| Fase 3 — Syncthing | 3 slices | ⏳ Pendente | ~12 horas |
| Fase 4 — Dashboard | 3 slices | ⏳ Pendente | ~10 horas |
| Fase 5 — Módulos | 8 slices | ⏳ Pendente | ~40 horas |
| Fase 6 — Polish | 6 slices | ⏳ Pendente | ~18 horas |
| **TOTAL** | **~30 slices** | **1 fase concluída** | **~102 horas** |

---

## Próximo Passo

Quando estiver pronto para codar:
1. Continue a sessão no Hermes
2. Pergunte: "Vamos prosseguir para o Slice 2.2?"
3. Eu vou implementar o próximo vertical slice

Boa implementação! 🚀
