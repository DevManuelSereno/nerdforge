import { useEffect, useState, useCallback } from "react";
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
import { FolderOpen, RefreshCw, CheckCircle2, XCircle, FileText, ChevronRight, ChevronDown } from "lucide-react";

interface VaultConfig {
  version: number;
  folder_map: Record<string, string>;
  folder_template: {
    name: string;
    folders: string[];
  };
}

interface VaultTreeNode {
  name: string;
  path: string;
  is_file: boolean;
  children: VaultTreeNode[];
}

interface IndexEntry {
  id: string;
  name: string;
  module: string;
  relative_path: string;
  kind: string;
  last_modified?: number;
  size?: number;
  title?: string;
  status?: string;
  tags?: string[];
}

interface VaultIndex {
  version: number;
  generated_at: string;
  root: string;
  entries: IndexEntry[];
}

function TreeNode({ node, depth = 0 }: { node: VaultTreeNode; depth?: number }) {
  const [expanded, setExpanded] = useState(depth < 1);

  const hasChildren = node.children.length > 0;
  const isDotNerdforge = node.name === ".nerdforge";

  if (isDotNerdforge && depth > 0) {
    return null;
  }

  return (
    <div className="select-none">
      <div
        className="flex items-center gap-1 rounded-md py-1 pr-2 text-sm hover:bg-muted/50"
        style={{ paddingLeft: `${depth * 16 + 4}px` }}
      >
        {hasChildren ? (
          <button
            type="button"
            onClick={() => setExpanded(!expanded)}
            className="flex h-5 w-5 shrink-0 items-center justify-center rounded text-muted-foreground hover:text-foreground"
          >
            {expanded ? <ChevronDown className="h-3.5 w-3.5" /> : <ChevronRight className="h-3.5 w-3.5" />}
          </button>
        ) : (
          <span className="h-5 w-5 shrink-0" />
        )}
        {node.is_file ? (
          <FileText className="h-4 w-4 shrink-0 text-muted-foreground" />
        ) : (
          <FolderOpen className="h-4 w-4 shrink-0 text-muted-foreground" />
        )}
        <span className="truncate" title={node.path}>
          {node.name}
        </span>
      </div>
      {expanded && hasChildren && (
        <div>
          {node.children.map((child) => (
            <TreeNode key={child.path} node={child} depth={depth + 1} />
          ))}
        </div>
      )}
    </div>
  );
}

export function VaultPage() {
  const { workspace } = useWorkspace();
  const [tree, setTree] = useState<VaultTreeNode | null>(null);
  const [vaultConfig, setVaultConfig] = useState<VaultConfig | null>(null);
  const [index, setIndex] = useState<VaultIndex | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadTree = useCallback(async () => {
    if (!workspace?.rootPath) return;

    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<VaultTreeNode>("scan_vault_command", {
        root: workspace.rootPath,
        maxDepth: 2,
      });
      setTree(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }, [workspace?.rootPath]);

  const loadVaultConfig = useCallback(async () => {
    if (!workspace?.rootPath) return;

    try {
      const result = await invoke<VaultConfig>("ensure_vault_config_command", {
        root: workspace.rootPath,
      });
      setVaultConfig(result);
    } catch (err) {
      console.error("Falha ao carregar vaultconfig:", err);
    }
  }, [workspace?.rootPath]);

  const loadIndex = useCallback(async () => {
    if (!workspace?.rootPath) return;

    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<VaultIndex>("get_index_command", {
        root: workspace.rootPath,
      });
      setIndex(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }, [workspace?.rootPath]);

  async function initializeStructure() {
    if (!workspace?.rootPath) return;

    setIsLoading(true);
    setError(null);
    try {
      await invoke("initialize_vault_command", {
        root: workspace.rootPath,
      });
      await loadTree();
      await loadVaultConfig();
      await loadIndex();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }

  async function applyTemplate() {
    if (!workspace?.rootPath || !vaultConfig) return;

    setIsLoading(true);
    setError(null);
    try {
      await invoke("initialize_vault_with_folders_command", {
        root: workspace.rootPath,
        folders: vaultConfig.folder_template.folders,
      });
      await loadTree();
      await loadIndex();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }

  async function rebuildIndex() {
    if (!workspace?.rootPath) return;

    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<VaultIndex>("rebuild_index_command", {
        root: workspace.rootPath,
      });
      setIndex(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }

  useEffect(() => {
    loadTree();
    loadVaultConfig();
    loadIndex();
  }, [workspace?.rootPath, loadTree, loadVaultConfig, loadIndex]);

  const suggestedFolders = vaultConfig?.folder_template.folders ?? [];
  const existingNames = new Set(tree?.children.map((c) => c.name) ?? []);
  const missingFolders = suggestedFolders.filter((f) => !existingNames.has(f));

  const fileEntries = index?.entries.filter((e) => e.kind === "file") ?? [];
  const groupedByModule = fileEntries.reduce<Record<string, IndexEntry[]>>((acc, entry) => {
    if (!acc[entry.module]) acc[entry.module] = [];
    acc[entry.module].push(entry);
    return acc;
  }, {});

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold tracking-tight flex items-center gap-2">
          <FolderOpen className="h-6 w-6" />
          Vault
        </h2>
        <p className="text-muted-foreground">
          Estrutura de pastas e índice do seu mundo.
        </p>
      </div>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>Estrutura atual do vault</CardTitle>
            <CardDescription>
              {workspace?.rootPath || "Nenhum workspace selecionado"}
            </CardDescription>
          </div>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="icon"
              onClick={loadTree}
              disabled={isLoading}
              title="Atualizar"
            >
              <RefreshCw className={`h-4 w-4 ${isLoading ? "animate-spin" : ""}`} />
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {error && (
            <div className="mb-4 rounded-md bg-destructive/10 p-3 text-sm text-destructive">
              {error}
            </div>
          )}

          {!tree && !error && (
            <p className="text-sm text-muted-foreground">
              Nenhuma pasta encontrada.
            </p>
          )}

          {tree && (
            <div className="rounded-md border p-2">
              <TreeNode node={tree} />
            </div>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>Sugestão de estrutura</CardTitle>
            <CardDescription>
              {vaultConfig
                ? `Template: ${vaultConfig.folder_template.name}`
                : "Nenhum template carregado"}
            </CardDescription>
          </div>
          <div className="flex gap-2">
            <Button
              variant="outline"
              onClick={applyTemplate}
              disabled={isLoading || !workspace?.rootPath || missingFolders.length === 0}
            >
              {isLoading ? (
                <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <FolderOpen className="mr-2 h-4 w-4" />
              )}
              Aplicar sugestão
            </Button>
            <Button
              variant="outline"
              onClick={initializeStructure}
              disabled={isLoading || !workspace?.rootPath}
            >
              Criar estrutura
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-1">
            {suggestedFolders.length === 0 && (
              <p className="text-sm text-muted-foreground">
                Nenhuma pasta sugerida.
              </p>
            )}
            {suggestedFolders.map((folder) => {
              const exists = existingNames.has(folder);
              return (
                <div
                  key={folder}
                  className="flex items-center justify-between rounded-md border px-3 py-2 text-sm"
                >
                  <div className="flex items-center gap-2">
                    <FolderOpen className="h-4 w-4 text-muted-foreground" />
                    <span className="font-mono">{folder}</span>
                  </div>
                  {exists ? (
                    <CheckCircle2 className="h-4 w-4 text-green-500" />
                  ) : (
                    <XCircle className="h-4 w-4 text-destructive" />
                  )}
                </div>
              );
            })}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>Índice do vault</CardTitle>
            <CardDescription>
              {index
                ? `Gerado em ${new Date(index.generated_at).toLocaleString("pt-BR")} — ${index.entries.length} entradas`
                : "Nenhum índice carregado"}
            </CardDescription>
          </div>
          <Button
            variant="outline"
            onClick={rebuildIndex}
            disabled={isLoading || !workspace?.rootPath}
          >
            Reconstruir índice
          </Button>
        </CardHeader>
        <CardContent>
          {Object.keys(groupedByModule).length === 0 ? (
            <p className="text-sm text-muted-foreground">
              Nenhum arquivo indexado. Adicione arquivos markdown nas pastas do vault e clique em "Reconstruir índice".
            </p>
          ) : (
            <div className="space-y-4">
              {Object.entries(groupedByModule).map(([module, moduleEntries]) => (
                <div key={module}>
                  <h4 className="mb-2 text-sm font-semibold uppercase tracking-wide text-muted-foreground">
                    {module}
                  </h4>
                  <div className="space-y-1">
                    {moduleEntries.map((entry) => (
                      <div
                        key={entry.id}
                        className="flex flex-col gap-2 rounded-md border px-3 py-2 text-sm"
                      >
                        <div className="flex items-center justify-between">
                          <div className="flex items-center gap-2">
                            <FileText className="h-4 w-4 text-muted-foreground" />
                            <span className="font-mono">{entry.name}</span>
                            {entry.title && (
                              <span className="font-medium text-foreground">
                                — {entry.title}
                              </span>
                            )}
                          </div>
                          <div className="flex items-center gap-2">
                            {entry.status && (
                              <span className="rounded-full bg-primary/10 px-2 py-0.5 text-xs text-primary">
                                {entry.status}
                              </span>
                            )}
                            {entry.size !== undefined && (
                              <span className="text-xs text-muted-foreground">
                                {(entry.size / 1024).toFixed(1)} KB
                              </span>
                            )}
                          </div>
                        </div>
                        <div className="flex items-center gap-1 pl-6">
                          <span className="text-xs text-muted-foreground">
                            {entry.relative_path}
                          </span>
                        </div>
                        {entry.tags && entry.tags.length > 0 && (
                          <div className="flex flex-wrap gap-1 pl-6">
                            {entry.tags.map((tag) => (
                              <span
                                key={tag}
                                className="rounded-md bg-muted px-1.5 py-0.5 text-xs text-muted-foreground"
                              >
                                #{tag}
                              </span>
                            ))}
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
