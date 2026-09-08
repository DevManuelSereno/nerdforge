import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useWorkspace } from "@/contexts/WorkspaceContext";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  AlertTriangle,
  RefreshCw,
  Loader2,
  ArrowLeft,
  Check,
  FileText,
  CheckCircle2,
} from "lucide-react";

interface SyncConflict {
  file_name: string;
  relative_path: string;
  absolute_path: string;
  size: number;
  last_modified?: number;
}

interface ConflictPair {
  original_path: string;
  conflict_path: string;
  original_content: string | null;
  conflict_content: string;
  original_name: string;
  conflict_name: string;
}

export function ConflictsPage() {
  const { workspace } = useWorkspace();
  const [conflicts, setConflicts] = useState<SyncConflict[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedConflict, setSelectedConflict] = useState<SyncConflict | null>(null);
  const [pair, setPair] = useState<ConflictPair | null>(null);
  const [pairLoading, setPairLoading] = useState(false);
  const [resolving, setResolving] = useState(false);
  const [resolved, setResolved] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const loadConflicts = useCallback(async () => {
    if (!workspace?.rootPath) return;
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<SyncConflict[]>("find_conflicts_command", {
        root: workspace.rootPath,
      });
      setConflicts(result);
    } catch (err) {
      setError(`Falha ao procurar conflitos: ${err}`);
    } finally {
      setLoading(false);
    }
  }, [workspace?.rootPath]);

  useEffect(() => {
    loadConflicts();
  }, [loadConflicts]);

  async function loadPair(conflict: SyncConflict) {
    setSelectedConflict(conflict);
    setPair(null);
    setResolved(null);
    setPairLoading(true);
    setError(null);
    try {
      const result = await invoke<ConflictPair>("load_conflict_pair_command", {
        conflictPath: conflict.absolute_path,
      });
      setPair(result);
    } catch (err) {
      setError(`Falha ao carregar conflito: ${err}`);
    } finally {
      setPairLoading(false);
    }
  }

  async function resolve(action: "keep_local" | "keep_remote" | "delete_conflict") {
    if (!selectedConflict) return;
    const labels: Record<string, string> = {
      keep_local: "manter a versão local",
      keep_remote: "manter a versão remota",
      delete_conflict: "descartar o arquivo de conflito",
    };
    const confirmed = window.confirm(
      `Tem certeza que deseja ${labels[action]}? Esta ação não pode ser desfeita.`
    );
    if (!confirmed) return;

    setResolving(true);
    setError(null);
    try {
      await invoke("resolve_conflict_command", {
        conflictPath: selectedConflict.absolute_path,
        action,
      });
      setResolved(selectedConflict.absolute_path);
      await loadConflicts();
      setSelectedConflict(null);
      setPair(null);
    } catch (err) {
      setError(`Falha ao resolver conflito: ${err}`);
    } finally {
      setResolving(false);
    }
  }

  if (!workspace?.rootPath) {
    return (
      <div className="flex h-full flex-col items-center justify-center text-center">
        <AlertTriangle className="h-12 w-12 text-muted-foreground" />
        <h2 className="mt-4 text-xl font-semibold">Nenhum projeto aberto</h2>
        <p className="text-sm text-muted-foreground">
          Abra um projeto para verificar conflitos de sincronização.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight flex items-center gap-2">
            <AlertTriangle className="h-6 w-6" />
            Conflitos de Sync
          </h2>
          <p className="text-muted-foreground">
            Resolva conflitos de sincronização do Syncthing.
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={loadConflicts} disabled={loading}>
          <RefreshCw className={`mr-2 h-4 w-4 ${loading ? "animate-spin" : ""}`} />
          Atualizar
        </Button>
      </div>

      {error && (
        <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">{error}</div>
      )}

      {resolved && (
        <Card className="border-green-600/20 bg-green-50 dark:bg-green-950/20">
          <CardContent className="flex items-center gap-2 p-4">
            <CheckCircle2 className="h-5 w-5 text-green-600 dark:text-green-400" />
            <p className="text-sm font-medium text-green-700 dark:text-green-300">
              Conflito resolvido com sucesso!
            </p>
          </CardContent>
        </Card>
      )}

      {!selectedConflict ? (
        <>
          {loading ? (
            <div className="flex items-center gap-2 text-muted-foreground">
              <Loader2 className="h-4 w-4 animate-spin" />
              Procurando conflitos...
            </div>
          ) : conflicts.length === 0 ? (
            <div className="rounded-lg border border-dashed border-border p-10 text-center">
              <CheckCircle2 className="mx-auto h-10 w-10 text-green-500" />
              <h3 className="mt-4 text-lg font-semibold">Nenhum conflito encontrado</h3>
              <p className="mt-1 text-sm text-muted-foreground">
                Seu vault está sincronizado sem conflitos.
              </p>
            </div>
          ) : (
            <div className="space-y-2">
              <p className="text-sm text-muted-foreground">
                {conflicts.length} conflito(s) encontrado(s). Clique para resolver.
              </p>
              <div className="grid gap-3">
                {conflicts.map((conflict) => (
                  <Card
                    key={conflict.absolute_path}
                    className="cursor-pointer border-yellow-600/20 transition-colors hover:border-yellow-600/50"
                    onClick={() => loadPair(conflict)}
                  >
                    <CardContent className="p-4">
                      <div className="flex items-start gap-3">
                        <AlertTriangle className="mt-0.5 h-5 w-5 text-yellow-600 dark:text-yellow-400" />
                        <div className="min-w-0 flex-1">
                          <p className="truncate font-medium">{conflict.file_name}</p>
                          <p className="truncate font-mono text-xs text-muted-foreground">
                            {conflict.relative_path}
                          </p>
                          <div className="mt-2 flex items-center gap-3 text-xs text-muted-foreground">
                            <span>{(conflict.size / 1024).toFixed(1)} KB</span>
                            {conflict.last_modified && (
                              <span>
                                {new Date(conflict.last_modified * 1000).toLocaleString("pt-BR")}
                              </span>
                            )}
                          </div>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </div>
          )}
        </>
      ) : (
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={() => { setSelectedConflict(null); setPair(null); }}>
              <ArrowLeft className="mr-2 h-4 w-4" />
              Voltar à lista
            </Button>
          </div>

          {pairLoading ? (
            <div className="flex items-center gap-2 text-muted-foreground">
              <Loader2 className="h-4 w-4 animate-spin" />
              Carregando arquivos...
            </div>
          ) : pair ? (
            <>
              <div className="flex items-start justify-between gap-4">
                <div>
                  <h3 className="text-lg font-semibold">{pair.conflict_name}</h3>
                  <p className="text-sm text-muted-foreground font-mono">
                    {selectedConflict.relative_path}
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => resolve("keep_local")}
                    disabled={resolving}
                  >
                    <Check className="mr-2 h-4 w-4" />
                    Manter Local
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => resolve("keep_remote")}
                    disabled={resolving}
                  >
                    <Check className="mr-2 h-4 w-4" />
                    Manter Remota
                  </Button>
                  <Button
                    variant="destructive"
                    size="sm"
                    onClick={() => resolve("delete_conflict")}
                    disabled={resolving}
                  >
                    {resolving ? (
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    ) : (
                      <AlertTriangle className="mr-2 h-4 w-4" />
                    )}
                    Descartar Conflito
                  </Button>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <Card>
                  <CardContent className="p-4">
                    <div className="mb-3 flex items-center gap-2">
                      <FileText className="h-4 w-4 text-green-600" />
                      <span className="text-sm font-medium">Local (atual)</span>
                      <Badge variant="outline" className="ml-auto text-xs">
                        {pair.original_name}
                      </Badge>
                    </div>
                    <pre className="max-h-96 overflow-auto whitespace-pre-wrap rounded-md bg-muted p-3 text-xs font-mono">
                      {pair.original_content ?? "(arquivo não encontrado)"}
                    </pre>
                  </CardContent>
                </Card>

                <Card className="border-yellow-600/20">
                  <CardContent className="p-4">
                    <div className="mb-3 flex items-center gap-2">
                      <AlertTriangle className="h-4 w-4 text-yellow-600" />
                      <span className="text-sm font-medium">Remota (conflito)</span>
                      <Badge variant="outline" className="ml-auto text-xs">
                        {pair.conflict_name}
                      </Badge>
                    </div>
                    <pre className="max-h-96 overflow-auto whitespace-pre-wrap rounded-md bg-yellow-50 dark:bg-yellow-950/20 p-3 text-xs font-mono">
                      {pair.conflict_content}
                    </pre>
                  </CardContent>
                </Card>
              </div>

              <div className="rounded-md bg-muted p-3 text-sm text-muted-foreground">
                <p>
                  <strong>Manter Local:</strong> descarta o arquivo de conflito e mantém o arquivo
                  atual como está.
                </p>
                <p className="mt-1">
                  <strong>Manter Remota:</strong> substitui o arquivo atual pelo conteúdo do
                  conflito e remove o arquivo de conflito.
                </p>
                <p className="mt-1">
                  <strong>Descartar Conflito:</strong> apenas remove o arquivo de conflito (igual a
                  "Manter Local").
                </p>
              </div>
            </>
          ) : (
            <p className="text-sm text-muted-foreground">Falha ao carregar o conflito.</p>
          )}
        </div>
      )}
    </div>
  );
}
