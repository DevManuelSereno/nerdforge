import {
  createContext,
  useContext,
  useState,
  ReactNode,
  useEffect,
  useCallback,
} from "react";
import { invoke } from "@tauri-apps/api/core";

export interface Workspace {
  name: string;
  rootPath: string;
}

interface AppConfig {
  workspace: {
    name: string;
    root_path: string;
  };
}

interface WorkspaceContextType {
  workspace: Workspace | null;
  setWorkspace: (workspace: Workspace | null) => void;
  isLoading: boolean;
  setIsLoading: (loading: boolean) => void;
  error: string | null;
  setError: (error: string | null) => void;
  isFirstRun: boolean | null;
  setIsFirstRun: (value: boolean) => void;
  loadWorkspace: () => Promise<void>;
  configPath: string;
}

const CONFIG_PATH = ".nerdforge/config.yaml";

const WorkspaceContext = createContext<WorkspaceContextType | undefined>(
  undefined,
);

function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  return Promise.race([
    promise,
    new Promise<never>((_, reject) =>
      setTimeout(() => reject(new Error("Tempo esgotado")), ms),
    ),
  ]);
}

export function WorkspaceProvider({ children }: { children: ReactNode }) {
  const [workspace, setWorkspace] = useState<Workspace | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isFirstRun, setIsFirstRun] = useState<boolean | null>(null);

  const loadWorkspace = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const firstRun = await withTimeout(
        invoke<boolean>("is_first_run", {
          path: CONFIG_PATH,
        }),
        5000,
      );
      setIsFirstRun(firstRun);

      if (!firstRun) {
        const config = await withTimeout(
          invoke<AppConfig>("load_config_command", {
            path: CONFIG_PATH,
          }),
          5000,
        );
        setWorkspace({
          name: config.workspace.name,
          rootPath: config.workspace.root_path,
        });
      }
    } catch (err) {
      setError(
        `Erro ao carregar workspace: ${err instanceof Error ? err.message : String(err)}`,
      );
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadWorkspace();
  }, [loadWorkspace]);

  return (
    <WorkspaceContext.Provider
      value={{
        workspace,
        setWorkspace,
        isLoading,
        setIsLoading,
        error,
        setError,
        isFirstRun,
        setIsFirstRun,
        loadWorkspace,
        configPath: CONFIG_PATH,
      }}
    >
      {children}
    </WorkspaceContext.Provider>
  );
}

export function useWorkspace() {
  const context = useContext(WorkspaceContext);
  if (!context) {
    throw new Error("useWorkspace must be used within a WorkspaceProvider");
  }
  return context;
}
