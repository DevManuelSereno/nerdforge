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
import { RefreshCw } from "lucide-react";

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

export function Dashboard() {
  const { workspace, isLoading, error } = useWorkspace();
  const [syncHealth, setSyncHealth] = useState<SyncthingHealth | null>(null);
  const [vaultFolder, setVaultFolder] = useState<VaultFolderStatus | null>(null);
  const [syncLoading, setSyncLoading] = useState(false);

  useEffect(() => {
    loadSyncStatus();
  }, []);

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
            <CardTitle className="text-sm font-medium">NPCs</CardTitle>
            <CardDescription>Módulo de personagens</CardDescription>
          </CardHeader>
          <CardContent>
            <Button variant="outline" size="sm" asChild>
              <a href="#/npc">Abrir NPCs</a>
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Lore</CardTitle>
            <CardDescription>Módulo de conhecimento</CardDescription>
          </CardHeader>
          <CardContent>
            <Button variant="outline" size="sm" asChild>
              <a href="#/lore">Abrir Lore</a>
            </Button>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
