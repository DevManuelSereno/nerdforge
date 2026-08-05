import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useWorkspace } from "@/contexts/WorkspaceContext";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Users,
  Plus,
  Wand2,
  FileText,
  Loader2,
  CheckCircle2,
  RefreshCw,
  User,
  FolderOpen,
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

interface NpcSummary {
  id: string;
  name: string;
  slug: string;
  relative_path: string;
  absolute_path: string;
  status?: string;
  tags: string[];
  last_modified?: number;
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

export function NPCPage() {
  const { workspace } = useWorkspace();
  const [npcs, setNpcs] = useState<NpcSummary[]>([]);
  const [npcsLoading, setNpcsLoading] = useState(false);
  const [templates, setTemplates] = useState<Template[]>([]);
  const [templatesLoading, setTemplatesLoading] = useState(false);
  const [name, setName] = useState("");
  const [creating, setCreating] = useState(false);
  const [mode, setMode] = useState<"dashboard" | "template" | "blank" | "hermes">("dashboard");
  const [selectedTemplate, setSelectedTemplate] = useState<Template | null>(null);
  const [result, setResult] = useState<ApplyTemplateResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const loadNpcs = useCallback(async () => {
    if (!workspace?.rootPath) return;
    setNpcsLoading(true);
    setError(null);
    try {
      const result = await invoke<NpcSummary[]>("list_npcs_command", {
        root: workspace.rootPath,
      });
      setNpcs(result);
    } catch (err) {
      setError(`Falha ao carregar NPCs: ${err}`);
    } finally {
      setNpcsLoading(false);
    }
  }, [workspace?.rootPath]);

  const loadTemplates = useCallback(async () => {
    if (!workspace?.rootPath) return;
    setTemplatesLoading(true);
    setError(null);
    try {
      const result = await invoke<Template[]>("list_templates_command", {
        root: workspace.rootPath,
        module: "npc",
      });
      setTemplates(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setTemplatesLoading(false);
    }
  }, [workspace?.rootPath]);

  useEffect(() => {
    loadNpcs();
  }, [loadNpcs]);

  useEffect(() => {
    if (workspace?.rootPath && mode === "template") {
      loadTemplates();
    }
  }, [workspace?.rootPath, mode, loadTemplates]);

  function resetFlow() {
    setMode("dashboard");
    setName("");
    setSelectedTemplate(null);
    setResult(null);
    setError(null);
  }

  function handleCreated() {
    resetFlow();
    loadNpcs();
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
        module: "npc",
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

  if (!workspace?.rootPath) {
    return (
      <div className="flex h-full flex-col items-center justify-center text-center">
        <Users className="h-12 w-12 text-muted-foreground" />
        <h2 className="mt-4 text-xl font-semibold">Nenhum projeto aberto</h2>
        <p className="text-sm text-muted-foreground">
          Abra ou crie um projeto para gerenciar NPCs.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight flex items-center gap-2">
            <Users className="h-6 w-6" />
            NPCs
          </h2>
          <p className="text-muted-foreground">
            Gerencie personagens não-jogadores do seu mundo.
          </p>
        </div>
        <div className="flex items-center gap-2">
          {mode === "dashboard" && (
            <>
              <Button
                variant="outline"
                size="sm"
                onClick={loadNpcs}
                disabled={npcsLoading}
              >
                <RefreshCw
                  className={`mr-2 h-4 w-4 ${npcsLoading ? "animate-spin" : ""}`}
                />
                Atualizar
              </Button>
              <Button size="sm" onClick={() => setMode("template")}>
                <Plus className="mr-2 h-4 w-4" />
                Novo NPC
              </Button>
            </>
          )}
          {mode !== "dashboard" && (
            <Button variant="outline" size="sm" onClick={resetFlow}>
              Voltar
            </Button>
          )}
        </div>
      </div>

      {error && (
        <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
          {error}
        </div>
      )}

      {mode === "dashboard" && (
        <>
          {result && (
            <Card className="border-green-600/20 bg-green-50 dark:bg-green-950/20">
              <CardContent className="space-y-2 p-4">
                <div className="flex items-center gap-2 text-green-600 dark:text-green-400">
                  <CheckCircle2 className="h-5 w-5" />
                  <p className="font-medium">NPC criado</p>
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

          {npcs.length === 0 ? (
            <div className="rounded-lg border border-dashed border-border p-10 text-center">
              <User className="mx-auto h-10 w-10 text-muted-foreground" />
              <h3 className="mt-4 text-lg font-semibold">Nenhum NPC ainda</h3>
              <p className="mt-1 text-sm text-muted-foreground">
                Crie seu primeiro NPC usando um template, o Hermes ou do zero.
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
              {npcs.map((npc) => (
                <Card
                  key={npc.id}
                  className="cursor-pointer transition-colors hover:border-primary/50"
                  onClick={() => console.log("selected npc", npc)}
                >
                  <CardContent className="p-4">
                    <div className="flex items-start gap-3">
                      <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-muted">
                        <User className="h-5 w-5 text-muted-foreground" />
                      </div>
                      <div className="min-w-0 flex-1">
                        <p className="truncate font-medium">{npc.name}</p>
                        <p className="text-xs text-muted-foreground">
                          {formatRelativeTime(npc.last_modified)}
                        </p>
                        {npc.status && (
                          <Badge variant="secondary" className="mt-2 text-xs">
                            {npc.status}
                          </Badge>
                        )}
                        {npc.tags.length > 0 && (
                          <div className="mt-2 flex flex-wrap gap-1">
                            {npc.tags.slice(0, 4).map((tag) => (
                              <Badge key={tag} variant="outline" className="text-xs">
                                {tag}
                              </Badge>
                            ))}
                            {npc.tags.length > 4 && (
                              <Badge variant="outline" className="text-xs">
                                +{npc.tags.length - 4}
                              </Badge>
                            )}
                          </div>
                        )}
                      </div>
                    </div>
                    <div className="mt-4 flex items-center gap-2 text-xs text-muted-foreground">
                      <FolderOpen className="h-3 w-3" />
                      <span className="truncate">{npc.relative_path}</span>
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
            <h3 className="text-lg font-semibold">Criar NPC a partir de template</h3>
            <p className="text-sm text-muted-foreground">
              Escolha um template e defina o nome do personagem.
            </p>
          </div>

          <div>
            <label className="mb-1.5 block text-sm font-medium">Nome do NPC</label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Ex: Kael Thornwood"
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
                        <p className="text-sm text-muted-foreground">
                          {template.description}
                        </p>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}

          {selectedTemplate && (
            <div className="rounded-md bg-muted p-3 text-sm">
              <p className="font-medium">Template selecionado: {selectedTemplate.name}</p>
              <p className="text-muted-foreground">
                Arquivos: {selectedTemplate.files.map((f) => f.relative_path).join(", ")}
              </p>
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
            <Button
              variant="outline"
              onClick={() => {
                setMode("blank");
                setSelectedTemplate(null);
              }}
            >
              Criar do zero
            </Button>
          </div>
        </div>
      )}

      {mode === "blank" && (
        <div className="space-y-4">
          <div>
            <h3 className="text-lg font-semibold">Criar NPC do zero</h3>
            <p className="text-sm text-muted-foreground">
              Crie uma estrutura mínima vazia para editar depois.
            </p>
          </div>

          <div>
            <label className="mb-1.5 block text-sm font-medium">Nome do NPC</label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Ex: Kael Thornwood"
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
              A geração com Hermes via interface do NPC será integrada no Slice 5.6.
              Use o Laboratório de Structured Output para testar a geração.
            </p>
          </div>
          <Button
            variant="outline"
            onClick={() => {
              window.location.hash = "#/structured-output";
            }}
          >
            <Wand2 className="mr-2 h-4 w-4" />
            Abrir Structured Output
          </Button>
        </div>
      )}
    </div>
  );
}
