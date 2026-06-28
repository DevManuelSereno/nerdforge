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
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { RefreshCw, Play, Square, FolderSync, Copy, Check, Plus, UserPlus } from "lucide-react";

interface SyncthingStatus {
  running: boolean;
  pid?: number;
  executable?: string;
  config_dir?: string;
  data_dir?: string;
  error?: string;
}

interface SyncthingHealth {
  healthy: boolean;
  latency_ms?: number;
  error?: string;
}

interface SyncthingSystemStatus {
  my_id: string;
  uptime_s: number;
  version: string;
}

interface FolderStatus {
  id: string;
  label: string;
  path: string;
  folder_type: string;
}

interface FolderConfigResult {
  configured: boolean;
  folder_id: string;
  path: string;
  error?: string;
}

interface DeviceInfo {
  device_id: string;
  name: string;
  addresses: string[];
  connected: boolean;
  introducer: boolean;
}

interface PendingDevice {
  device_id: string;
  name: string;
  address: string;
}

interface AddDeviceResult {
  added: boolean;
  device_id: string;
  error?: string;
}

export function SettingsPage() {
  const { workspace, setWorkspace } = useWorkspace();
  const [name, setName] = useState(workspace?.name || "");
  const [path, setPath] = useState(workspace?.rootPath || "");
  const [logPath, setLogPath] = useState<string | null>(null);
  const [syncthing, setSyncthing] = useState<SyncthingStatus | null>(null);
  const [syncthingLoading, setSyncthingLoading] = useState(false);
  const [health, setHealth] = useState<SyncthingHealth | null>(null);
  const [systemStatus, setSystemStatus] = useState<SyncthingSystemStatus | null>(null);
  const [folders, setFolders] = useState<FolderStatus[]>([]);
  const [vaultConfigured, setVaultConfigured] = useState(false);
  const [configuringVault, setConfiguringVault] = useState(false);
  const [deviceId, setDeviceId] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [newDeviceId, setNewDeviceId] = useState("");
  const [newDeviceName, setNewDeviceName] = useState("");
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
  const [pendingDevices, setPendingDevices] = useState<PendingDevice[]>([]);
  const [addingDevice, setAddingDevice] = useState(false);
  const [vaultDevices, setVaultDevices] = useState<string[]>([]);

  useEffect(() => {
    loadSyncthingStatus();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function handleSave() {
    setWorkspace({ name, rootPath: path });
  }

  async function loadLogPath() {
    try {
      const result = await invoke<string>("get_log_path");
      setLogPath(result);
    } catch (err) {
      setLogPath(String(err));
    }
  }

  async function loadSyncthingStatus() {
    setSyncthingLoading(true);
    try {
      const result = await invoke<SyncthingStatus>("syncthing_status_command");
      setSyncthing(result);
      if (result.running && result.config_dir) {
        await loadHealth();
      }
    } catch (err) {
      setSyncthing({ running: false, error: String(err) });
    } finally {
      setSyncthingLoading(false);
    }
  }

  async function loadHealth() {
    try {
      const result = await invoke<SyncthingHealth>("syncthing_health_check_command");
      setHealth(result);
      if (result.healthy) {
        await loadSystemStatus();
        await loadFolders();
        await loadVaultFolderStatus();
        await loadDeviceId();
        await loadDevices();
        await loadPendingDevices();
        await loadVaultDevices();
      }
    } catch (err) {
      setHealth({ healthy: false, error: String(err) });
    }
  }

  async function loadSystemStatus() {
    try {
      const result = await invoke<SyncthingSystemStatus>("syncthing_system_status_command");
      setSystemStatus(result);
    } catch {
      setSystemStatus(null);
    }
  }

  async function loadFolders() {
    try {
      const result = await invoke<FolderStatus[]>("syncthing_folders_command");
      setFolders(result);
    } catch {
      setFolders([]);
    }
  }

  async function loadVaultFolderStatus() {
    try {
      const result = await invoke<FolderStatus | null>("get_vault_folder_status_command");
      setVaultConfigured(!!result);
    } catch {
      setVaultConfigured(false);
    }
  }

  async function loadDeviceId() {
    try {
      const result = await invoke<string>("get_device_id_command");
      setDeviceId(result);
    } catch {
      setDeviceId(null);
    }
  }

  async function loadDevices() {
    try {
      const result = await invoke<DeviceInfo[]>("list_devices_command");
      setDevices(result);
    } catch {
      setDevices([]);
    }
  }

  async function loadPendingDevices() {
    try {
      const result = await invoke<PendingDevice[]>("list_pending_devices_command");
      setPendingDevices(result);
    } catch {
      setPendingDevices([]);
    }
  }

  async function copyDeviceId() {
    if (!deviceId) return;
    try {
      await navigator.clipboard.writeText(deviceId);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      setHealth({ healthy: false, error: String(err) });
    }
  }

  async function loadVaultDevices() {
    try {
      const result = await invoke<string[]>("list_vault_devices_command");
      setVaultDevices(result);
    } catch {
      setVaultDevices([]);
    }
  }

  async function shareVaultWithDevice(deviceId: string) {
    try {
      const result = await invoke<FolderConfigResult>("share_vault_with_device_command", {
        deviceId,
      });
      if (result.configured) {
        await loadVaultDevices();
      } else {
        setHealth({ healthy: false, error: result.error || "Falha ao compartilhar vault" });
      }
    } catch (err) {
      setHealth({ healthy: false, error: String(err) });
    }
  }

  async function addNewDevice() {
    if (!newDeviceId.trim()) return;
    setAddingDevice(true);
    try {
      const result = await invoke<AddDeviceResult>("add_device_command", {
        deviceId: newDeviceId.trim(),
        deviceName: newDeviceName.trim() || "Dispositivo NerdForge",
      });
      if (result.added) {
        setNewDeviceId("");
        setNewDeviceName("");
        await loadDevices();
        await shareVaultWithDevice(result.device_id);
      } else {
        setHealth({ healthy: false, error: result.error || "Falha ao adicionar dispositivo" });
      }
    } catch (err) {
      setHealth({ healthy: false, error: String(err) });
    } finally {
      setAddingDevice(false);
    }
  }

  async function acceptPendingDevice(deviceId: string) {
    try {
      const result = await invoke<AddDeviceResult>("accept_device_command", {
        deviceId,
      });
      if (result.added) {
        await loadDevices();
        await loadPendingDevices();
      } else {
        setHealth({ healthy: false, error: result.error || "Falha ao aceitar dispositivo" });
      }
    } catch (err) {
      setHealth({ healthy: false, error: String(err) });
    }
  }

  async function configureVault() {
    if (!workspace?.rootPath) return;
    setConfiguringVault(true);
    try {
      const result = await invoke<FolderConfigResult>("configure_vault_folder_command", {
        root: workspace.rootPath,
      });
      setVaultConfigured(result.configured);
      await loadFolders();
    } catch (err) {
      setHealth({ healthy: false, error: String(err) });
    } finally {
      setConfiguringVault(false);
    }
  }

  async function startSyncthing() {
    if (!workspace?.rootPath) return;
    setSyncthingLoading(true);
    try {
      const result = await invoke<SyncthingStatus>("start_syncthing_command", {
        root: workspace.rootPath,
      });
      setSyncthing(result);
    } catch (err) {
      setSyncthing({ running: false, error: String(err) });
    } finally {
      setSyncthingLoading(false);
    }
  }

  async function stopSyncthing() {
    setSyncthingLoading(true);
    try {
      const result = await invoke<SyncthingStatus>("stop_syncthing_command");
      setSyncthing(result);
    } catch (err) {
      setSyncthing({ running: false, error: String(err) });
    } finally {
      setSyncthingLoading(false);
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold tracking-tight">Configurações</h2>
        <p className="text-muted-foreground">
          Configure seu workspace e preferências.
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Workspace</CardTitle>
          <CardDescription>
            Nome e local dos arquivos do seu mundo.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <label htmlFor="workspace-name" className="text-sm font-medium">
              Nome
            </label>
            <Input
              id="workspace-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>
          <div className="space-y-2">
            <label htmlFor="workspace-path" className="text-sm font-medium">
              Caminho do vault
            </label>
            <Input
              id="workspace-path"
              value={path}
              onChange={(e) => setPath(e.target.value)}
            />
          </div>
          <Button onClick={handleSave}>Salvar workspace</Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Logs</CardTitle>
          <CardDescription>Visualize o caminho do arquivo de log.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Button variant="outline" onClick={loadLogPath}>
            Carregar caminho do log
          </Button>
          {logPath && (
            <p className="break-all rounded-md bg-muted p-2 text-xs font-mono">
              {logPath}
            </p>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Syncthing</CardTitle>
          <CardDescription>
            Controle o processo local de sincronização do vault.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2">
              <div
                className={`h-3 w-3 rounded-full ${
                  syncthing?.running ? "bg-green-500" : "bg-red-500"
                }`}
              />
              <span className="text-sm font-medium">
                {syncthing?.running ? "Rodando" : "Parado"}
              </span>
            </div>
            {syncthing?.pid && (
              <span className="text-sm text-muted-foreground">PID: {syncthing.pid}</span>
            )}
          </div>

          {syncthing?.error && (
            <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
              {syncthing.error}
            </div>
          )}

          {syncthing?.executable && (
            <p className="text-xs text-muted-foreground">
              Executável: {syncthing.executable}
            </p>
          )}

          {syncthing?.config_dir && (
            <p className="text-xs text-muted-foreground">
              Config: {syncthing.config_dir}
            </p>
          )}

          {health && (
            <div className="flex items-center gap-2 text-sm">
              <div
                className={`h-3 w-3 rounded-full ${
                  health.healthy ? "bg-green-500" : "bg-yellow-500"
                }`}
              />
              <span>
                API: {health.healthy ? "OK" : "Indisponível"}
                {health.latency_ms !== undefined && ` (${health.latency_ms}ms)`}
              </span>
            </div>
          )}

          {systemStatus && (
            <div className="space-y-1 text-sm">
              <p>
                <span className="text-muted-foreground">Versão:</span>{" "}
                {systemStatus.version}
              </p>
              <p>
                <span className="text-muted-foreground">ID:</span>{" "}
                <span className="font-mono text-xs">{systemStatus.my_id}</span>
              </p>
              <p>
                <span className="text-muted-foreground">Uptime:</span>{" "}
                {Math.floor(systemStatus.uptime_s / 60)} min
              </p>
            </div>
          )}

          {folders.length > 0 && (
            <div className="space-y-2">
              <h4 className="text-sm font-medium">Folders configurados</h4>
              <div className="space-y-1">
                {folders.map((folder) => (
                  <div
                    key={folder.id}
                    className="rounded-md border px-3 py-2 text-sm"
                  >
                    <div className="font-medium">{folder.label || folder.id}</div>
                    <div className="font-mono text-xs text-muted-foreground">
                      {folder.path}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          <div className="flex flex-wrap gap-2">
            <Button
              onClick={startSyncthing}
              disabled={syncthingLoading || !workspace?.rootPath}
            >
              <Play className="mr-2 h-4 w-4" />
              Iniciar
            </Button>
            <Button
              variant="outline"
              onClick={stopSyncthing}
              disabled={syncthingLoading}
            >
              <Square className="mr-2 h-4 w-4" />
              Parar
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={loadSyncthingStatus}
              disabled={syncthingLoading}
            >
              <RefreshCw
                className={`h-4 w-4 ${syncthingLoading ? "animate-spin" : ""}`}
              />
            </Button>
            <Button
              variant={vaultConfigured ? "outline" : "default"}
              onClick={configureVault}
              disabled={configuringVault || !syncthing?.running || !workspace?.rootPath}
            >
              <FolderSync className="mr-2 h-4 w-4" />
              {vaultConfigured ? "Vault sincronizado" : "Sincronizar este vault"}
            </Button>
          </div>

          {deviceId && (
            <div className="space-y-2 rounded-md border p-3">
              <h4 className="text-sm font-medium">Seu Device ID</h4>
              <p className="text-xs text-muted-foreground">
                Compartilhe este ID com outro dispositivo para emparelhar.
              </p>
              <div className="flex items-center gap-2">
                <code className="flex-1 rounded-md bg-muted p-2 text-xs font-mono break-all">
                  {deviceId}
                </code>
                <Button variant="outline" size="icon" onClick={copyDeviceId}>
                  {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
                </Button>
              </div>
            </div>
          )}

          {health?.healthy && (
            <div className="space-y-2 rounded-md border p-3">
              <h4 className="text-sm font-medium">Adicionar dispositivo</h4>
              <p className="text-xs text-muted-foreground">
                Cole o Device ID do outro dispositivo e dê um nome.
              </p>
              <Input
                placeholder="Nome do dispositivo"
                value={newDeviceName}
                onChange={(e) => setNewDeviceName(e.target.value)}
              />
              <Input
                placeholder="Device ID do outro dispositivo"
                value={newDeviceId}
                onChange={(e) => setNewDeviceId(e.target.value)}
              />
              <Button
                onClick={addNewDevice}
                disabled={addingDevice || !newDeviceId.trim()}
              >
                <UserPlus className="mr-2 h-4 w-4" />
                Adicionar dispositivo
              </Button>
            </div>
          )}

          {pendingDevices.length > 0 && (
            <div className="space-y-2">
              <h4 className="text-sm font-medium">Pedidos pendentes</h4>
              <div className="space-y-1">
                {pendingDevices.map((device) => (
                  <div
                    key={device.device_id}
                    className="flex items-center justify-between rounded-md border px-3 py-2 text-sm"
                  >
                    <div>
                      <div className="font-medium">{device.name || "Dispositivo desconhecido"}</div>
                      <div className="font-mono text-xs text-muted-foreground">
                        {device.device_id}
                      </div>
                    </div>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => acceptPendingDevice(device.device_id)}
                    >
                      <Plus className="mr-2 h-4 w-4" />
                      Aceitar
                    </Button>
                  </div>
                ))}
              </div>
            </div>
          )}

          {devices.length > 0 && (
            <div className="space-y-2">
              <h4 className="text-sm font-medium">Dispositivos sincronizados</h4>
              <div className="space-y-1">
                {devices.map((device) => {
                  const hasVaultAccess = vaultDevices.includes(device.device_id);
                  return (
                    <div
                      key={device.device_id}
                      className="flex items-center justify-between rounded-md border px-3 py-2 text-sm"
                    >
                      <div>
                        <div className="font-medium">{device.name || device.device_id}</div>
                        <div className="font-mono text-xs text-muted-foreground">
                          {device.device_id}
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        <div
                          className={`h-2 w-2 rounded-full ${
                            device.connected ? "bg-green-500" : "bg-gray-400"
                          }`}
                        />
                        <span className="text-xs text-muted-foreground">
                          {device.connected ? "Conectado" : "Desconectado"}
                        </span>
                        {hasVaultAccess ? (
                          <span className="rounded-full bg-green-100 px-2 py-0.5 text-xs text-green-700">
                            Vault compartilhado
                          </span>
                        ) : (
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => shareVaultWithDevice(device.device_id)}
                          >
                            <FolderSync className="mr-2 h-3 w-3" />
                            Compartilhar vault
                          </Button>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
