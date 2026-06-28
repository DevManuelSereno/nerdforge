import { useState } from "react";
import { useNavigate } from "react-router-dom";
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
import { FolderOpen, Sparkles } from "lucide-react";

interface AppConfig {
  workspace: {
    name: string;
    root_path: string;
  };
  modules: {
    npc: { enabled: boolean };
    lore: { enabled: boolean };
  };
  sync: {
    engine: string;
    embedded: boolean;
    api_key: string;
  };
  hermes: {
    mode: string;
    context_window: number;
    auto_summarize: boolean;
  };
}

export function Onboarding() {
  const navigate = useNavigate();
  const { setWorkspace, setIsFirstRun, configPath, loadWorkspace } =
    useWorkspace();
  const [name, setName] = useState("Meu Vault");
  const [rootPath, setRootPath] = useState(
    "C:\\Users\\GAMER\\Documents\\NerdForge",
  );
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleCreateWorkspace() {
    setIsLoading(true);
    setError(null);
    try {
      const config: AppConfig = {
        workspace: {
          name: name.trim(),
          root_path: rootPath.trim(),
        },
        modules: {
          npc: { enabled: true },
          lore: { enabled: true },
        },
        sync: {
          engine: "syncthing",
          embedded: true,
          api_key: "auto-generated",
        },
        hermes: {
          mode: "cli",
          context_window: 20,
          auto_summarize: true,
        },
      };

      await invoke("save_config_command", {
        path: configPath,
        config,
      });

      await invoke("initialize_vault_command", {
        root: rootPath.trim(),
      });

      setWorkspace({ name: config.workspace.name, rootPath });
      setIsFirstRun(false);
      await loadWorkspace();
      navigate("/");
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-4">
      <div className="w-full max-w-xl space-y-8">
        <div className="text-center">
          <div className="mb-4 flex justify-center">
            <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-primary text-primary-foreground">
              <Sparkles className="h-8 w-8" />
            </div>
          </div>
          <h1 className="text-4xl font-bold tracking-tight">Bem-vindo ao NerdForge</h1>
          <p className="mt-2 text-muted-foreground">
            Configure seu primeiro workspace para começar a construir mundos.
          </p>
        </div>

        <Card>
          <CardHeader>
            <CardTitle>Criar workspace</CardTitle>
            <CardDescription>
              O workspace é a pasta onde todos os seus arquivos serão armazenados.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <label htmlFor="workspace-name" className="text-sm font-medium">
                Nome do workspace
              </label>
              <Input
                id="workspace-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Ex: Meu Mundo de RPG"
              />
            </div>

            <div className="space-y-2">
              <label htmlFor="workspace-path" className="text-sm font-medium">
                Caminho do vault
              </label>
              <div className="flex gap-2">
                <Input
                  id="workspace-path"
                  value={rootPath}
                  onChange={(e) => setRootPath(e.target.value)}
                  placeholder="C:\\Users\\...\\Documents\\NerdForge"
                />
                <Button
                  variant="outline"
                  size="icon"
                  onClick={() => setRootPath("C:\\Users\\GAMER\\Documents\\NerdForge")}
                  title="Usar pasta padrão"
                >
                  <FolderOpen className="h-4 w-4" />
                </Button>
              </div>
            </div>

            {error && (
              <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
                {error}
              </div>
            )}

            <Button
              className="w-full"
              onClick={handleCreateWorkspace}
              disabled={isLoading || !name.trim() || !rootPath.trim()}
            >
              {isLoading ? "Criando..." : "Criar workspace e começar"}
            </Button>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
