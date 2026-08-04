import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useWorkspace } from "@/contexts/WorkspaceContext";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import { Textarea } from "@/components/ui/textarea";
import { Loader2, Wand2, AlertCircle, CheckCircle2, FileText } from "lucide-react";

interface StructuredOutputData {
  module: string;
  data: Record<string, unknown>;
  raw_reply: string;
  extracted_yaml: string;
  markdown: string;
  visual_dna_yaml: string | null;
  prompt_md: string | null;
  validation_errors: string[];
}

type OutputModule = "npc" | "lore";

export function StructuredOutputLab() {
  const { workspace } = useWorkspace();
  const [module, setModule] = useState<OutputModule>("npc");
  const [prompt, setPrompt] = useState("");
  const [context, setContext] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<StructuredOutputData | null>(null);
  const [activeTab, setActiveTab] = useState<
    "preview" | "yaml" | "raw" | "visual" | "prompt"
  >("preview");

  async function handleGenerate() {
    if (!prompt.trim()) return;
    setLoading(true);
    setError(null);
    setResult(null);

    try {
      const response = await invoke<StructuredOutputData>(
        "generate_structured_output_command",
        {
          request: {
            module,
            prompt: prompt.trim(),
            context: context.trim() || null,
          },
        }
      );
      setResult(response);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  if (!workspace?.rootPath) {
    return (
      <div className="flex h-full flex-col items-center justify-center text-center">
        <FileText className="h-12 w-12 text-muted-foreground" />
        <h2 className="mt-4 text-xl font-semibold">Nenhum projeto aberto</h2>
        <p className="text-sm text-muted-foreground">
          Abra ou crie um projeto para testar geração estruturada.
        </p>
      </div>
    );
  }

  const hasErrors = result && result.validation_errors.length > 0;
  const isValid = result && result.validation_errors.length === 0;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold tracking-tight flex items-center gap-2">
          <Wand2 className="h-6 w-6" />
          Laboratório de Structured Output
        </h2>
        <p className="text-muted-foreground">
          Teste a geração de YAML estruturado com delimitadores. Pré-requisito
          para o Slice 5.6 (Módulo NPC).
        </p>
      </div>

      <Card>
        <CardContent className="space-y-4 p-4">
          <div className="flex gap-2">
            <Button
              variant={module === "npc" ? "default" : "outline"}
              onClick={() => setModule("npc")}
              type="button"
            >
              NPC
            </Button>
            <Button
              variant={module === "lore" ? "default" : "outline"}
              onClick={() => setModule("lore")}
              type="button"
            >
              Lore
            </Button>
          </div>

          <div>
            <label className="mb-1.5 block text-sm font-medium">
              Solicitação
            </label>
            <Textarea
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              placeholder={
                module === "npc"
                  ? "Ex: Crie um NPC mercador de informações para uma cidade portuária sombria."
                  : "Ex: Escreva um artigo de lore sobre a queda da cidade de Valdris."
              }
              rows={4}
            />
          </div>

          <div>
            <label className="mb-1.5 block text-sm font-medium">
              Contexto adicional (opcional)
            </label>
            <Input
              value={context}
              onChange={(e) => setContext(e.target.value)}
              placeholder="Ex: O mundo é dark fantasy, tecnologia medieval, magia rara."
            />
          </div>

          <Button
            onClick={handleGenerate}
            disabled={!prompt.trim() || loading}
            className="w-full"
          >
            {loading ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Gerando com Hermes...
              </>
            ) : (
              <>
                <Wand2 className="mr-2 h-4 w-4" />
                Gerar estruturado
              </>
            )}
          </Button>
        </CardContent>
      </Card>

      {error && (
        <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
          {error}
        </div>
      )}

      {result && (
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            {hasErrors ? (
              <>
                <AlertCircle className="h-5 w-5 text-destructive" />
                <span className="text-sm font-medium">
                  YAML gerado com erros de validação
                </span>
              </>
            ) : isValid ? (
              <>
                <CheckCircle2 className="h-5 w-5 text-green-600" />
                <span className="text-sm font-medium text-green-600">
                  Estrutura válida
                </span>
              </>
            ) : null}
          </div>

          {hasErrors && (
            <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
              <ul className="list-inside list-disc space-y-1">
                {result.validation_errors.map((err, idx) => (
                  <li key={idx}>{err}</li>
                ))}
              </ul>
            </div>
          )}

          <div className="flex gap-2 overflow-x-auto pb-1">
            {[
              { key: "preview", label: "Preview Markdown" },
              { key: "yaml", label: "YAML Extraído" },
              { key: "visual", label: "Visual DNA" },
              { key: "prompt", label: "Prompt Visual" },
              { key: "raw", label: "Resposta Bruta" },
            ].map((tab) => (
              <Button
                key={tab.key}
                variant={activeTab === tab.key ? "default" : "outline"}
                size="sm"
                onClick={() => setActiveTab(tab.key as typeof activeTab)}
              >
                {tab.label}
              </Button>
            ))}
          </div>

          <Card>
            <CardContent className="p-4">
              {activeTab === "preview" && (
                <pre className="whitespace-pre-wrap text-sm">{result.markdown}</pre>
              )}
              {activeTab === "yaml" && (
                <pre className="whitespace-pre-wrap text-sm font-mono">
                  {result.extracted_yaml}
                </pre>
              )}
              {activeTab === "visual" && (
                <pre className="whitespace-pre-wrap text-sm font-mono">
                  {result.visual_dna_yaml || "Não disponível para este módulo."}
                </pre>
              )}
              {activeTab === "prompt" && (
                <pre className="whitespace-pre-wrap text-sm font-mono">
                  {result.prompt_md || "Não disponível para este módulo."}
                </pre>
              )}
              {activeTab === "raw" && (
                <pre className="whitespace-pre-wrap text-sm font-mono">
                  {result.raw_reply}
                </pre>
              )}
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
}
