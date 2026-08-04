import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useWorkspace } from "@/contexts/WorkspaceContext";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import { Users, Plus, Wand2, FileText, Loader2, CheckCircle2 } from "lucide-react";

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

export function NPCPage() {
  const { workspace } = useWorkspace();
  const [templates, setTemplates] = useState<Template[]>([]);
  const [templatesLoading, setTemplatesLoading] = useState(false);
  const [name, setName] = useState("");
  const [creating, setCreating] = useState(false);
  const [mode, setMode] = useState<"idle" | "template" | "blank" | "hermes">("idle");
  const [selectedTemplate, setSelectedTemplate] = useState<Template | null>(null);
  const [result, setResult] = useState<ApplyTemplateResult | null>(null);
  const [error, setError] = useState<string | null>(null);

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
    if (workspace?.rootPath && mode === "template") {
      loadTemplates();
    }
  }, [workspace?.rootPath, mode, loadTemplates]);

  function resetFlow() {
    setMode("idle");
    setName("");
    setSelectedTemplate(null);
    setResult(null);
    setError(null);
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
        {mode === "idle" && (
          <Button onClick={() => setMode("template")}>
            <Plus className="mr-2 h-4 w-4" />
            Novo NPC
          </Button>
        )}
      </div>

      {mode === "idle" && !result && (
        <div className="rounded-lg border border-dashed border-border p-8 text-center">
          <p className="text-muted-foreground">
            Nenhum NPC criado ainda. Clique em "Novo NPC" para começar.
          </p>
        </div>
      )}

      {mode === "template" && (
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            <Button variant="outline" onClick={resetFlow}>
              Voltar
            </Button>
            <h3 className="text-lg font-semibold">Criar NPC a partir de template</h3>
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
          <div className="flex items-center gap-2">
            <Button variant="outline" onClick={resetFlow}>
              Voltar
            </Button>
            <h3 className="text-lg font-semibold">Criar NPC do zero</h3>
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
          <div className="flex items-center gap-2">
            <Button variant="outline" onClick={resetFlow}>
              Voltar
            </Button>
            <h3 className="text-lg font-semibold">Gerar com Hermes</h3>
          </div>
          <p className="text-sm text-muted-foreground">
            A geração com Hermes via interface do NPC será integrada no Slice 5.6.
            Use o Laboratório de Structured Output para testar a geração.
          </p>
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

      {error && (
        <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
          {error}
        </div>
      )}

      {result && (
        <Card>
          <CardContent className="space-y-3 p-4">
            <div className="flex items-center gap-2 text-green-600">
              <CheckCircle2 className="h-5 w-5" />
              <p className="font-medium">NPC criado</p>
            </div>
            <p className="text-sm text-muted-foreground">
              Caminho: <code className="rounded bg-muted px-1">{result.entity_path}</code>
            </p>
            <p className="text-sm text-muted-foreground">
              Arquivos: {result.files_created.length}
            </p>
            <Button variant="outline" onClick={resetFlow}>
              Criar outro
            </Button>
          </CardContent>
        </Card>
      )}

      {mode === "idle" && (
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => setMode("template")}>
            <Plus className="mr-2 h-4 w-4" />
            Usar template
          </Button>
          <Button variant="outline" onClick={() => setMode("hermes")}>
            <Wand2 className="mr-2 h-4 w-4" />
            Gerar com Hermes
          </Button>
          <Button variant="outline" onClick={() => setMode("blank")}>
            <FileText className="mr-2 h-4 w-4" />
            Criar do zero
          </Button>
        </div>
      )}
    </div>
  );
}
