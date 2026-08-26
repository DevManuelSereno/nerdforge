import { useWorkspace } from "@/contexts/WorkspaceContext";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import {
  RefreshCw,
  FolderOpen,
  UserPlus,
  Clock,
  FileText,
  HardDrive,
  AlertTriangle,
} from "lucide-react";

interface SyncthingHealth {
  healthy: boolean;
  error?: string;
}

interface VaultFolderStatus {
  id: string;
  label: string;
  path: string;
  folder_type: string;
}

interface ProjectInfo {
  name: string;
  path: string;
  created_at?: number;
  updated_at?: number;
  file_count: number;
  is_current: boolean;
}

interface ActivityEvent {
  timestamp: number;
  module: string;
  title: string;
  path: string;
  event_type: string;
}

interface DashboardData {
  projects: ProjectInfo[];
  recent_activities: ActivityEvent[];
}

interface SyncConflict {
  file_name: string;
  relative_path: string;
  absolute_path: string;
  size: number;
  last_modified?: number;
}

export function Dashboard() {
  const { workspace, isLoading, error, setWorkspace } = useWorkspace();
  const [syncHealth, setSyncHealth] = useState<SyncthingHealth | null>(null);
  const [vaultFolder, setVaultFolder] = useState<VaultFolderStatus | null>(null);
  const [syncLoading, setSyncLoading] = useState(false);
  const [dashboardData, setDashboardData] = useState<DashboardData | null>(null);
  const [dashboardLoading, setDashboardLoading] = useState(false);
  const [conflicts, setConflicts] = useState<SyncConflict[]>([]);
  const [conflictsLoading, setConflictsLoading] = useState(false);

  useEffect(() => {
    loadSyncStatus();
    loadDashboardData();
    loadConflicts();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (workspace?.rootPath) {
      loadDashboardData();
      loadConflicts();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [workspace?.rootPath]);

  async function loadSyncStatus() {
    setSyncLoading(true);
    try {
      const health = await invoke<SyncthingHealth>("syncthing_health_check_command");
      setSyncHealth(health);
      if (health.healthy) {
        const folder = await invoke<VaultFolderStatus | null>("get_vault_folder_status_command");
        setVaultFolder(folder);
      }
    } catch (err) {
      setSyncHealth({ healthy: false, error: String(err) });
      setVaultFolder(null);
    } finally {
      setSyncLoading(false);
    }
  }

  async function loadDashboardData() {
    if (!workspace?.rootPath) return;
    setDashboardLoading(true);
    try {
      const data = await invoke<DashboardData>("get_dashboard_data_command", {
        root: workspace.rootPath,
        limit: 10,
      });
      setDashboardData(data);
    } catch (err) {
      console.error("Falha ao carregar dashboard:", err);
    } finally {
      setDashboardLoading(false);
    }
  }

  function openProject(path: string) {
    const name = path.split(/[\\/]/).pop() || path;
    setWorkspace({ name, rootPath: path });
  }

  async function loadConflicts() {
    if (!workspace?.rootPath) return;
    setConflictsLoading(true);
    try {
      const result = await invoke<SyncConflict[]>("find_conflicts_command", {
        root: workspace.rootPath,
      });
      setConflicts(result);
    } catch (err) {
      console.error("Falha ao procurar conflitos:", err);
    } finally {
      setConflictsLoading(false);
    }
  }

  function formatDate(timestamp?: number): string {
    if (!timestamp) return "-";
    return new Date(timestamp * 1000).toLocaleDateString("pt-BR", {
      day: "2-digit",
      month: "short",
      year: "numeric",
    });
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

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold tracking-tight">Dashboard</h2>
        <p className="text-muted-foreground">
          Visão geral do seu mundo e projetos.
        </p>
      </div>

      {isLoading && <p>Carregando workspace...</p>}
      {error && (
        <div className="rounded-md bg-destructive/10 p-4 text-destructive">
          {error}
        </div>
      )}

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Workspace</CardTitle>
            <CardDescription>
              {workspace ? workspace.name : "Nenhum workspace carregado"}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-xs text-muted-foreground">
              {workspace ? workspace.rootPath : "-"}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm font-medium">Sincronização</CardTitle>
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={loadSyncStatus}
                disabled={syncLoading}
              >
                <RefreshCw
                  className={`h-3 w-3 ${syncLoading ? "animate-spin" : ""}`}
                />
              </Button>
            </div>
            <CardDescription>
              {syncHealth
                ? syncHealth.healthy
                  ? "Syncthing conectado"
                  : syncHealth.error || "Syncthing indisponível"
                : "Verificando..."}
            </CardDescription>
          </CardHeader>
          <CardContent>
            {vaultFolder ? (
              <div className="space-y-1 text-sm">
                <p className="text-green-600 dark:text-green-400">
                  Vault configurado para sync
                </p>
                <p className="text-xs text-muted-foreground font-mono">
                  {vaultFolder.path}
                </p>
              </div>
            ) : syncHealth?.healthy ? (
              <p className="text-sm text-yellow-600 dark:text-yellow-400">
                Vault ainda não configurado
              </p>
            ) : (
              <p className="text-xs text-muted-foreground">
                Inicie o Syncthing em Configurações
              </p>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Ações rápidas</CardTitle>
            <CardDescription>Atalhos comuns</CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            <Button variant="outline" size="sm" className="w-full justify-start" asChild>
              <a href="#/vault">
                <FolderOpen className="mr-2 h-4 w-4" />
                Abrir vault
              </a>
            </Button>
            <Button variant="outline" size="sm" className="w-full justify-start" asChild>
              <a href="#/settings">
                <UserPlus className="mr-2 h-4 w-4" />
                Adicionar dispositivo
              </a>
            </Button>
          </CardContent>
        </Card>

        <Card className="md:col-span-2 lg:col-span-2">
          <CardHeader className="pb-2">
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm font-medium">Projetos</CardTitle>
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={loadDashboardData}
                disabled={dashboardLoading}
              >
                <RefreshCw
                  className={`h-3 w-3 ${dashboardLoading ? "animate-spin" : ""}`}
                />
              </Button>
            </div>
            <CardDescription>
              {dashboardData?.projects.length || 0} projetos encontrados
            </CardDescription>
          </CardHeader>
          <CardContent>
            {dashboardData && dashboardData.projects.length > 0 ? (
              <div className="grid gap-2 sm:grid-cols-2">
                {dashboardData.projects.map((project) => (
                  <div
                    key={project.path}
                    className="flex flex-col gap-2 rounded-md border p-3"
                  >
                    <div className="flex items-start justify-between gap-2">
                      <div>
                        <p className="font-medium">{project.name}</p>
                        <p className="text-xs text-muted-foreground line-clamp-1">
                          {project.path}
                        </p>
                      </div>
                      {project.is_current && (
                        <span className="rounded-full bg-primary/10 px-2 py-0.5 text-xs">
                          Atual
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-3 text-xs text-muted-foreground">
                      <span className="flex items-center gap-1">
                        <FileText className="h-3 w-3" />
                        {project.file_count} arquivos
                      </span>
                      <span className="flex items-center gap-1">
                        <Clock className="h-3 w-3" />
                        {formatDate(project.updated_at)}
                      </span>
                    </div>
                    {!project.is_current && (
                      <Button
                        variant="outline"
                        size="sm"
                        className="mt-1 w-full"
                        onClick={() => openProject(project.path)}
                      >
                        <FolderOpen className="mr-2 h-3 w-3" />
                        Abrir
                      </Button>
                    )}
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">
                {dashboardLoading
                  ? "Carregando projetos..."
                  : "Nenhum projeto encontrado ainda."}
              </p>
            )}
          </CardContent>
        </Card>

        {conflicts.length > 0 && (
          <Card className="border-yellow-600/30 bg-yellow-50 dark:bg-yellow-950/20">
            <CardHeader className="pb-2">
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center gap-2 text-sm font-medium">
                  <AlertTriangle className="h-4 w-4 text-yellow-600 dark:text-yellow-400" />
                  Conflitos de Sync
                </CardTitle>
                <div className="flex items-center gap-2">
                  {conflictsLoading && (
                    <RefreshCw className="h-3 w-3 animate-spin text-muted-foreground" />
                  )}
                  <span className="rounded-full bg-yellow-600/20 px-2 py-0.5 text-xs font-medium text-yellow-700 dark:text-yellow-300">
                    {conflicts.length}
                  </span>
                </div>
              </div>
              <CardDescription>
                Arquivos com conflito de sincronização detectados
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                {conflicts.slice(0, 5).map((conflict) => (
                  <div
                    key={conflict.absolute_path}
                    className="rounded-md border border-yellow-600/20 px-3 py-2 text-sm"
                  >
                    <div className="truncate font-medium">{conflict.file_name}</div>
                    <div className="truncate font-mono text-xs text-muted-foreground">
                      {conflict.relative_path}
                    </div>
                  </div>
                ))}
                {conflicts.length > 5 && (
                  <p className="text-xs text-muted-foreground">
                    +{conflicts.length - 5} conflito(s) adicional(is)
                  </p>
                )}
              </div>
            </CardContent>
          </Card>
        )}

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Atividades recentes</CardTitle>
            <CardDescription>Últimas alterações no vault</CardDescription>
          </CardHeader>
          <CardContent>
            {dashboardData && dashboardData.recent_activities.length > 0 ? (
              <div className="space-y-3">
                {dashboardData.recent_activities.map((activity, idx) => (
                  <div key={idx} className="flex items-start gap-2 text-sm">
                    <HardDrive className="mt-0.5 h-4 w-4 text-muted-foreground" />
                    <div className="flex-1">
                      <p className="font-medium">{activity.title}</p>
                      <p className="text-xs text-muted-foreground capitalize">
                        {activity.module} • {formatRelativeTime(activity.timestamp)}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">
                {dashboardLoading
                  ? "Carregando atividades..."
                  : "Nenhuma atividade recente."}
              </p>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
