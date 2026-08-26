import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useWorkspace } from "@/contexts/WorkspaceContext";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import {
  Swords,
  Plus,
  Wand2,
  FileText,
  Loader2,
  CheckCircle2,
  RefreshCw,
  FolderOpen,
  ArrowLeft,
  Trash2,
  Copy,
  Check,
  Palette,
  Terminal,
  ExternalLink,
} from "lucide-react";

interface TemplateFile {
  relative_path: string;
  content: string;
}

interface Template {
  name: string;
  module: string;
  description: string;
  files: TemplateFile[];
}

interface ApplyTemplateResult {
  entity_path: string;
  files_created: string[];
}

interface EntitySummary {
  id: string;
  name: string;
  slug: string;
  relative_path: string;
  absolute_path: string;
  status?: string;
  tags: string[];
  last_modified?: number;
  category?: string | null;
}

interface EntityFiles {
  index_md: string;
  visual_dna_yaml?: string;
  prompt_md?: string;
}

function formatRelativeTime(timestamp?: number): string {
  if (!timestamp) return "-";
  const seconds = Math.floor((Date.now() - timestamp * 1000) / 1000);
  if (seconds < 60) return "agora";
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes} min atrás`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours} h atrás`;
  const days = Math.floor(hours / 24);
  return `${days} d atrás`;
}

const MODULE = "faction";
const MODULE_LABEL = "Facção";

export function FactionsPage() {
  const { workspace } = useWorkspace();
  const [entities, setEntities] = useState<EntitySummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [templates, setTemplates] = useState<Template[]>([]);
  const [templatesLoading, setTemplatesLoading] = useState(false);
  const [name, setName] = useState("");
  const [creating, setCreating] = useState(false);
  const [mode, setMode] = useState<"dashboard" | "template" | "blank" | "hermes" | "view">("dashboard");
  const [selectedTemplate, setSelectedTemplate] = useState<Template | null>(null);
  const [selectedEntity, setSelectedEntity] = useState<EntitySummary | null>(null);
  const [entityFiles, setEntityFiles] = useState<EntityFiles | null>(null);
  const [filesLoading, setFilesLoading] = useState(false);
  const [activeTab, setActiveTab] = useState<"lore" | "visual" | "prompt">("lore");
  const [copied, setCopied] = useState(false);
  const [obsidianAvailable, setObsidianAvailable] = useState<boolean | null>(null);
  const [result, setResult] = useState<ApplyTemplateResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const loadEntities = useCallback(async () => {
    if (!workspace?.rootPath) return;
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<EntitySummary[]>("list_entities_command", {
        root: workspace.rootPath,
        module: MODULE,
      });
      setEntities(result);
    } catch (err) {
      setError(`Falha ao carregar facções: ${err}`);
    } finally {
      setLoading(false);
    }
  }, [workspace?.rootPath]);

  const loadTemplates = useCallback(async () => {
    if (!workspace?.rootPath) return;
    setTemplatesLoading(true);
    setError(null);
    try {
      const result = await invoke<Template[]>("list_templates_command", {
        root: workspace.rootPath,
        module: MODULE,
      });
      setTemplates(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setTemplatesLoading(false);
    }
  }, [workspace?.rootPath]);

  const checkObsidian = useCallback(async () => {
    try {
      const available = await invoke<boolean>("is_obsidian_available_command");
      setObsidianAvailable(available);
    } catch {
      setObsidianAvailable(false);
    }
  }, []);

  const loadEntityFiles = useCallback(async (entity: EntitySummary) => {
    if (!workspace?.rootPath) return;
    setFilesLoading(true);
    setError(null);
    try {
      const result = await invoke<EntityFiles>("load_entity_files_command", {
        root: workspace.rootPath,
        relativePath: entity.relative_path,
      });
      setEntityFiles(result);
    } catch (err) {
      setError(`Falha ao carregar arquivos: ${err}`);
    } finally {
      setFilesLoading(false);
    }
  }, [workspace?.rootPath]);

  useEffect(() => {
    loadEntities();
  }, [loadEntities]);

  useEffect(() => {
    if (workspace?.rootPath && mode === "template") {
      loadTemplates();
    }
  }, [workspace?.rootPath, mode, loadTemplates]);

  useEffect(() => {
    if (mode === "view" && selectedEntity) {
      loadEntityFiles(selectedEntity);
      checkObsidian();
    }
  }, [mode, selectedEntity, loadEntityFiles, checkObsidian]);

  function resetFlow() {
    setMode("dashboard");
    setName("");
    setSelectedTemplate(null);
    setSelectedEntity(null);
    setEntityFiles(null);
    setResult(null);
    setError(null);
    setActiveTab("lore");
    setCopied(false);
    setObsidianAvailable(null);
  }

  function handleCreated() {
    resetFlow();
    loadEntities();
  }

  async function handleCreateWithTemplate() {
    if (!workspace?.rootPath || !selectedTemplate || !name.trim()) return;
    setCreating(true);
    setError(null);
    try {
      const response = await invoke<ApplyTemplateResult>("apply_template_command", {
        root: workspace.rootPath,
        template: selectedTemplate,
        name: name.trim(),
      });
      setResult(response);
      handleCreated();
    } catch (err) {
      setError(String(err));
    } finally {
      setCreating(false);
    }
  }

  async function handleCreateBlank() {
    if (!workspace?.rootPath || !name.trim()) return;
    setCreating(true);
    setError(null);
    try {
      const response = await invoke<ApplyTemplateResult>("create_blank_entity_command", {
        root: workspace.rootPath,
        module: MODULE,
        name: name.trim(),
      });
      setResult(response);
      handleCreated();
    } catch (err) {
      setError(String(err));
    } finally {
      setCreating(false);
    }
  }

  async function handleDeleteEntity() {
    if (!selectedEntity || !workspace?.rootPath) return;
    const confirmed = window.confirm(
      `Tem certeza que deseja excluir "${selectedEntity.name}"? Esta ação não pode ser desfeita.`
    );
    if (!confirmed) return;
    setError(null);
    try {
      await invoke("remove_dir", { path: selectedEntity.absolute_path });
      handleCreated();
    } catch (err) {
      setError(`Falha ao excluir: ${err}`);
    }
  }

  async function handleCopyPrompt() {
    if (!entityFiles?.prompt_md) return;
    try {
      await navigator.clipboard.writeText(entityFiles.prompt_md);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      setError(`Falha ao copiar: ${err}`);
    }
  }

  async function handleOpenInObsidian() {
    if (!selectedEntity?.absolute_path) return;
    setError(null);
    try {
      await invoke("open_in_obsidian_command", { path: selectedEntity.absolute_path });
    } catch (err) {
      setError(`Falha ao abrir no Obsidian: ${err}`);
    }
  }

  function handleSelectEntity(entity: EntitySummary) {
    setSelectedEntity(entity);
    setMode("view");
    setActiveTab("lore");
    setResult(null);
    setError(null);
  }

  if (!workspace?.rootPath) {
    return (
      <div className="flex h-full flex-col items-center justify-center text-center">
        <Swords className="h-12 w-12 text-muted-foreground" />
        <h2 className="mt-4 text-xl font-semibold">Nenhum projeto aberto</h2>
        <p className="text-sm text-muted-foreground">
          Abra ou crie um projeto para gerenciar facções.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight flex items-center gap-2">
            <Swords className="h-6 w-6" />
            Facções
          </h2>
          <p className="text-muted-foreground">
            Gerencie grupos, guildas e organizações do mundo.
          </p>
        </div>
        <div className="flex items-center gap-2">
          {mode === "dashboard" && (
            <>
              <Button variant="outline" size="sm" onClick={loadEntities} disabled={loading}>
                <RefreshCw className={`mr-2 h-4 w-4 ${loading ? "animate-spin" : ""}`} />
                Atualizar
              </Button>
              <Button size="sm" onClick={() => setMode("template")}>
                <Plus className="mr-2 h-4 w-4" />
                Nova Facção
              </Button>
            </>
          )}
          {mode !== "dashboard" && (
            <Button variant="outline" size="sm" onClick={resetFlow}>
              <ArrowLeft className="mr-2 h-4 w-4" />
              Voltar
            </Button>
          )}
        </div>
      </div>

      {error && (
        <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">{error}</div>
      )}

      {mode === "dashboard" && (
        <>
          {result && (
            <Card className="border-green-600/20 bg-green-50 dark:bg-green-950/20">
              <CardContent className="space-y-2 p-4">
                <div className="flex items-center gap-2 text-green-600 dark:text-green-400">
                  <CheckCircle2 className="h-5 w-5" />
                  <p className="font-medium">{MODULE_LABEL} criada</p>
                </div>
                <p className="text-sm text-muted-foreground">
                  Caminho: <code className="rounded bg-muted px-1">{result.entity_path}</code>
                </p>
                <p className="text-sm text-muted-foreground">
                  Arquivos: {result.files_created.length}
                </p>
              </CardContent>
            </Card>
          )}

          {entities.length === 0 ? (
            <div className="rounded-lg border border-dashed border-border p-10 text-center">
              <Swords className="mx-auto h-10 w-10 text-muted-foreground" />
              <h3 className="mt-4 text-lg font-semibold">Nenhuma facção ainda</h3>
              <p className="mt-1 text-sm text-muted-foreground">
                Crie sua primeira facção usando um template, o Hermes ou do zero.
              </p>
              <div className="mt-4 flex justify-center gap-2">
                <Button onClick={() => setMode("template")}>
                  <Plus className="mr-2 h-4 w-4" />
                  Usar template
                </Button>
                <Button variant="outline" onClick={() => setMode("blank")}>
                  <FileText className="mr-2 h-4 w-4" />
                  Do zero
                </Button>
              </div>
            </div>
          ) : (
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {entities.map((entity) => (
                <Card
                  key={entity.id}
                  className="cursor-pointer transition-colors hover:border-primary/50"
                  onClick={() => handleSelectEntity(entity)}
                >
                  <CardContent className="p-4">
                    <div className="flex items-start gap-3">
                      <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-muted">
                        <Swords className="h-5 w-5 text-muted-foreground" />
                      </div>
                      <div className="min-w-0 flex-1">
                        <p className="truncate font-medium">{entity.name}</p>
                        <p className="text-xs text-muted-foreground">
                          {formatRelativeTime(entity.last_modified)}
                        </p>
                        {entity.status && (
                          <Badge variant="secondary" className="mt-2 text-xs">
                            {entity.status}
                          </Badge>
                        )}
                        {entity.tags.length > 0 && (
                          <div className="mt-2 flex flex-wrap gap-1">
                            {entity.tags.slice(0, 4).map((tag) => (
                              <Badge key={tag} variant="outline" className="text-xs">
                                {tag}
                              </Badge>
                            ))}
                            {entity.tags.length > 4 && (
                              <Badge variant="outline" className="text-xs">
                                +{entity.tags.length - 4}
                              </Badge>
                            )}
                          </div>
                        )}
                      </div>
                    </div>
                    <div className="mt-4 flex items-center gap-2 text-xs text-muted-foreground">
                      <FolderOpen className="h-3 w-3" />
                      <span className="truncate">{entity.relative_path}</span>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </>
      )}

      {mode === "template" && (
        <div className="space-y-4">
          <div>
            <h3 className="text-lg font-semibold">Criar facção a partir de template</h3>
            <p className="text-sm text-muted-foreground">
              Escolha um template e defina o nome.
            </p>
          </div>
          <div>
            <label className="mb-1.5 block text-sm font-medium">Nome da facção</label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Ex: Ordem da Lua Negra"
            />
          </div>
          {templatesLoading ? (
            <div className="flex items-center gap-2 text-muted-foreground">
              <Loader2 className="h-4 w-4 animate-spin" />
              Carregando templates...
            </div>
          ) : templates.length === 0 ? (
            <p className="text-sm text-muted-foreground">Nenhum template disponível.</p>
          ) : (
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {templates.map((template) => (
                <Card
                  key={template.name}
                  className={`cursor-pointer transition-colors ${
                    selectedTemplate?.name === template.name
                      ? "border-primary"
                      : "hover:border-muted-foreground"
                  }`}
                  onClick={() => setSelectedTemplate(template)}
                >
                  <CardContent className="p-4">
                    <div className="flex items-start gap-3">
                      <FileText className="mt-0.5 h-5 w-5 text-muted-foreground" />
                      <div>
                        <p className="font-medium">{template.name}</p>
                        <p className="text-sm text-muted-foreground">{template.description}</p>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
          <div className="flex gap-2">
            <Button
              onClick={handleCreateWithTemplate}
              disabled={!name.trim() || !selectedTemplate || creating}
            >
              {creating ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Plus className="mr-2 h-4 w-4" />
              )}
              Criar com template
            </Button>
            <Button variant="outline" onClick={() => { setMode("blank"); setSelectedTemplate(null); }}>
              Criar do zero
            </Button>
          </div>
        </div>
      )}

      {mode === "blank" && (
        <div className="space-y-4">
          <div>
            <h3 className="text-lg font-semibold">Criar facção do zero</h3>
            <p className="text-sm text-muted-foreground">
              Crie uma estrutura mínima vazia para editar depois.
            </p>
          </div>
          <div>
            <label className="mb-1.5 block text-sm font-medium">Nome da facção</label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Ex: Ordem da Lua Negra"
            />
          </div>
          <Button onClick={handleCreateBlank} disabled={!name.trim() || creating}>
            {creating ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <FileText className="mr-2 h-4 w-4" />
            )}
            Criar estrutura vazia
          </Button>
        </div>
      )}

      {mode === "hermes" && (
        <div className="space-y-4">
          <div>
            <h3 className="text-lg font-semibold">Gerar com Hermes</h3>
            <p className="text-sm text-muted-foreground">
              Use o Laboratório de Structured Output para gerar facções.
            </p>
          </div>
          <Button variant="outline" onClick={() => { window.location.hash = "#/structured-output"; }}>
            <Wand2 className="mr-2 h-4 w-4" />
            Abrir Structured Output
          </Button>
        </div>
      )}

      {mode === "view" && selectedEntity && (
        <div className="space-y-4">
          <div className="flex items-start justify-between gap-4">
            <div className="flex items-start gap-4">
              <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-md bg-muted">
                <Swords className="h-6 w-6 text-muted-foreground" />
              </div>
              <div>
                <h3 className="text-xl font-semibold">{selectedEntity.name}</h3>
                <div className="mt-1 flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
                  <span className="flex items-center gap-1">
                    <FolderOpen className="h-3 w-3" />
                    {selectedEntity.relative_path}
                  </span>
                  {selectedEntity.status && (
                    <Badge variant="outline" className="text-xs">{selectedEntity.status}</Badge>
                  )}
                </div>
                {selectedEntity.tags.length > 0 && (
                  <div className="mt-2 flex flex-wrap gap-1">
                    {selectedEntity.tags.map((tag) => (
                      <Badge key={tag} variant="outline" className="text-xs">{tag}</Badge>
                    ))}
                  </div>
                )}
              </div>
            </div>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={handleOpenInObsidian}
                disabled={obsidianAvailable !== true}
                title={
                  obsidianAvailable === false
                    ? "Obsidian não encontrado"
                    : obsidianAvailable === null
                      ? "Verificando Obsidian..."
                      : "Abrir no Obsidian"
                }
              >
                <ExternalLink className="mr-2 h-4 w-4" />
                Abrir no Obsidian
              </Button>
              <Button variant="destructive" size="sm" onClick={handleDeleteEntity}>
                <Trash2 className="mr-2 h-4 w-4" />
                Excluir
              </Button>
            </div>
          </div>

          <div className="flex gap-1 border-b border-border">
            <button
              onClick={() => setActiveTab("lore")}
              className={`flex items-center gap-2 px-4 py-2 text-sm font-medium transition-colors ${
                activeTab === "lore"
                  ? "border-b-2 border-primary text-primary"
                  : "text-muted-foreground hover:text-foreground"
              }`}
            >
              <Swords className="h-4 w-4" />
              Conteúdo
            </button>
            <button
              onClick={() => setActiveTab("visual")}
              className={`flex items-center gap-2 px-4 py-2 text-sm font-medium transition-colors ${
                activeTab === "visual"
                  ? "border-b-2 border-primary text-primary"
                  : "text-muted-foreground hover:text-foreground"
              }`}
            >
              <Palette className="h-4 w-4" />
              Visual DNA
            </button>
            <button
              onClick={() => setActiveTab("prompt")}
              className={`flex items-center gap-2 px-4 py-2 text-sm font-medium transition-colors ${
                activeTab === "prompt"
                  ? "border-b-2 border-primary text-primary"
                  : "text-muted-foreground hover:text-foreground"
              }`}
            >
              <Terminal className="h-4 w-4" />
              Prompt
            </button>
          </div>

          {filesLoading ? (
            <div className="flex items-center gap-2 text-muted-foreground">
              <Loader2 className="h-4 w-4 animate-spin" />
              Carregando arquivos...
            </div>
          ) : (
            <>
              {activeTab === "lore" && (
                <div className="max-w-none rounded-md border border-border p-4">
                  <div className="space-y-4 leading-relaxed text-foreground">
                    <ReactMarkdown
                      remarkPlugins={[remarkGfm]}
                      components={{
                        h1: ({ children }) => <h1 className="text-2xl font-bold">{children}</h1>,
                        h2: ({ children }) => <h2 className="text-xl font-semibold">{children}</h2>,
                        h3: ({ children }) => <h3 className="text-lg font-semibold">{children}</h3>,
                        p: ({ children }) => <p className="text-foreground/90">{children}</p>,
                        ul: ({ children }) => <ul className="list-disc space-y-1 pl-5">{children}</ul>,
                        ol: ({ children }) => <ol className="list-decimal space-y-1 pl-5">{children}</ol>,
                        li: ({ children }) => <li>{children}</li>,
                        a: ({ children, href }) => (
                          <a href={href} className="text-primary underline hover:text-primary/80">{children}</a>
                        ),
                        code: ({ children }) => (
                          <code className="rounded bg-muted px-1 py-0.5 text-sm font-mono">{children}</code>
                        ),
                        pre: ({ children }) => (
                          <pre className="overflow-auto rounded-md bg-muted p-3 text-sm font-mono">{children}</pre>
                        ),
                        blockquote: ({ children }) => (
                          <blockquote className="border-l-4 border-muted-foreground/30 pl-4 italic text-muted-foreground">{children}</blockquote>
                        ),
                      }}
                    >
                      {entityFiles?.index_md ?? "Nenhum conteúdo disponível."}
                    </ReactMarkdown>
                  </div>
                </div>
              )}

              {activeTab === "visual" && (
                <div className="rounded-md border border-border bg-muted/50 p-4">
                  {entityFiles?.visual_dna_yaml ? (
                    <pre className="whitespace-pre-wrap text-sm font-mono">{entityFiles.visual_dna_yaml}</pre>
                  ) : (
                    <p className="text-sm text-muted-foreground">Nenhum visual-dna.yaml disponível.</p>
                  )}
                </div>
              )}

              {activeTab === "prompt" && (
                <div className="space-y-3">
                  {entityFiles?.prompt_md ? (
                    <>
                      <div className="flex items-center justify-between">
                        <p className="text-sm text-muted-foreground">Texto do prompt para geração com IA.</p>
                        <Button variant="outline" size="sm" onClick={handleCopyPrompt}>
                          {copied ? (
                            <><Check className="mr-2 h-4 w-4" />Copiado</>
                          ) : (
                            <><Copy className="mr-2 h-4 w-4" />Copiar</>
                          )}
                        </Button>
                      </div>
                      <pre className="whitespace-pre-wrap rounded-md border border-border bg-muted/50 p-4 text-sm font-mono">
                        {entityFiles.prompt_md}
                      </pre>
                    </>
                  ) : (
                    <p className="text-sm text-muted-foreground">Nenhum prompt.md disponível.</p>
                  )}
                </div>
              )}
            </>
          )}
        </div>
      )}
    </div>
  );
}
