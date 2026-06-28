import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useWorkspace } from "@/contexts/WorkspaceContext";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { FolderOpen, Plus, Trash2, Clock, FileText, RefreshCw } from "lucide-react";

interface ProjectInfo {
  name: string;
  path: string;
  created_at?: number;
  updated_at?: number;
  file_count: number;
  is_current: boolean;
}

export function ProjectsPage() {
  const { workspace, setWorkspace } = useWorkspace();
  const [projects, setProjects] = useState<ProjectInfo[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadProjects();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function loadProjects() {
    setLoading(true);
    try {
      const data = await invoke<ProjectInfo[]>("list_projects_command", {
        currentRoot: workspace?.rootPath ?? null,
      });
      setProjects(data);
    } catch (err) {
      console.error("Falha ao carregar projetos:", err);
    } finally {
      setLoading(false);
    }
  }

  function openProject(path: string) {
    const name = path.split(/[\\/]/).pop() || path;
    setWorkspace({ name, rootPath: path });
  }

  function formatDate(timestamp?: number): string {
    if (!timestamp) return "-";
    return new Date(timestamp * 1000).toLocaleDateString("pt-BR", {
      day: "2-digit",
      month: "short",
      year: "numeric",
    });
  }

  async function deleteProject(path: string) {
    if (!window.confirm("Tem certeza que deseja deletar este projeto?\n\nEsta ação não pode ser desfeita.")) {
      return;
    }
    try {
      await invoke("remove_dir", { path });
      if (workspace?.rootPath === path) {
        setWorkspace(null);
      }
      await loadProjects();
    } catch (err) {
      console.error("Falha ao deletar projeto:", err);
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Projetos</h2>
          <p className="text-muted-foreground">
            Gerencie seus mundos e campanhas.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="icon"
            onClick={loadProjects}
            disabled={loading}
          >
            <RefreshCw className={`h-4 w-4 ${loading ? "animate-spin" : ""}`} />
          </Button>
          <Button asChild>
            <a href="#/projects/new">
              <Plus className="mr-2 h-4 w-4" />
              Novo projeto
            </a>
          </Button>
        </div>
      </div>

      {projects.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12 text-center">
            <FolderOpen className="h-12 w-12 text-muted-foreground" />
            <h3 className="mt-4 text-lg font-medium">Nenhum projeto encontrado</h3>
            <p className="text-sm text-muted-foreground">
              Crie seu primeiro projeto para começar a construir seu mundo.
            </p>
            <Button className="mt-4" asChild>
              <a href="#/projects/new">
                <Plus className="mr-2 h-4 w-4" />
                Criar projeto
              </a>
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {projects.map((project) => (
            <Card
              key={project.path}
              className={project.is_current ? "border-primary" : undefined}
            >
              <CardHeader className="pb-2">
                <div className="flex items-start justify-between gap-2">
                  <div>
                    <CardTitle className="text-base">{project.name}</CardTitle>
                    <CardDescription className="line-clamp-1">
                      {project.path}
                    </CardDescription>
                  </div>
                  {project.is_current && (
                    <span className="rounded-full bg-primary/10 px-2 py-0.5 text-xs">
                      Atual
                    </span>
                  )}
                </div>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center gap-4 text-xs text-muted-foreground">
                  <span className="flex items-center gap-1">
                    <FileText className="h-3 w-3" />
                    {project.file_count} arquivos
                  </span>
                  <span className="flex items-center gap-1">
                    <Clock className="h-3 w-3" />
                    {formatDate(project.updated_at)}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  {!project.is_current && (
                    <Button
                      variant="outline"
                      size="sm"
                      className="flex-1"
                      onClick={() => openProject(project.path)}
                    >
                      <FolderOpen className="mr-2 h-3 w-3" />
                      Abrir
                    </Button>
                  )}
                  <Button
                    variant="outline"
                    size="icon"
                    className="text-destructive hover:bg-destructive/10"
                    onClick={() => deleteProject(project.path)}
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
