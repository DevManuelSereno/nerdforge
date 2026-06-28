import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useWorkspace } from "@/contexts/WorkspaceContext";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ArrowLeft, BookOpen, Dices, Sparkles, Loader2 } from "lucide-react";

interface ProjectInfo {
  name: string;
  path: string;
  file_count: number;
  is_current: boolean;
}

const templates = [
  {
    id: "rpg",
    label: "Campanha de RPG",
    description: "NPCs, Lore, Facções, Locais e Itens",
    icon: Dices,
  },
  {
    id: "book",
    label: "Livro / Romance",
    description: "Foco em Lore, Cenas e Linha do tempo",
    icon: BookOpen,
  },
  {
    id: "custom",
    label: "Personalizado",
    description: "Todos os módulos disponíveis",
    icon: Sparkles,
  },
];

export function NewProjectPage() {
  const { setWorkspace } = useWorkspace();
  const [name, setName] = useState("");
  const [template, setTemplate] = useState("rpg");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!name.trim()) return;

    setLoading(true);
    setError(null);
    try {
      const project = await invoke<ProjectInfo>("create_project_command", {
        name: name.trim(),
        parentDir: "documents",
        template,
      });
      setWorkspace({ name: project.name, rootPath: project.path });
      window.location.hash = "#/";
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <div className="flex items-center gap-2">
        <Button variant="ghost" size="icon" asChild>
          <a href="#/projects">
            <ArrowLeft className="h-4 w-4" />
          </a>
        </Button>
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Novo projeto</h2>
          <p className="text-muted-foreground">
            Crie um novo mundo ou campanha.
          </p>
        </div>
      </div>

      {error && (
        <div className="rounded-md bg-destructive/10 p-4 text-sm text-destructive">
          {error}
        </div>
      )}

      <form onSubmit={handleSubmit} className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Nome do projeto</CardTitle>
            <CardDescription>
              Escolha um nome único para o seu mundo.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Ex: Aventuras em Aethermoor"
              disabled={loading}
              autoFocus
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Template</CardTitle>
            <CardDescription>
              Selecione o tipo de projeto.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid gap-3 sm:grid-cols-3">
              {templates.map((t) => {
                const Icon = t.icon;
                return (
                  <button
                    key={t.id}
                    type="button"
                    onClick={() => setTemplate(t.id)}
                    disabled={loading}
                    className={`flex flex-col items-center gap-2 rounded-md border p-4 text-left transition-colors ${
                      template === t.id
                        ? "border-primary bg-primary/5"
                        : "hover:bg-muted"
                    }`}
                  >
                    <Icon className="h-6 w-6" />
                    <div className="text-center">
                      <p className="text-sm font-medium">{t.label}</p>
                      <p className="text-xs text-muted-foreground">
                        {t.description}
                      </p>
                    </div>
                  </button>
                );
              })}
            </div>
          </CardContent>
        </Card>

        <div className="flex justify-end gap-3">
          <Button variant="outline" asChild disabled={loading}>
            <a href="#/projects">Cancelar</a>
          </Button>
          <Button type="submit" disabled={!name.trim() || loading}>
            {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Criar projeto
          </Button>
        </div>
      </form>
    </div>
  );
}
