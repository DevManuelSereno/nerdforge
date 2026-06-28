import { RouterProvider, createHashRouter, Navigate } from "react-router-dom";
import { WorkspaceProvider, useWorkspace } from "@/contexts/WorkspaceContext";
import { AppLayout } from "@/layouts/AppLayout";
import { Dashboard } from "@/pages/Dashboard";
import { NPCPage } from "@/pages/NPCPage";
import { LorePage } from "@/pages/LorePage";
import { VaultPage } from "@/pages/VaultPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { Onboarding } from "@/pages/Onboarding";

function AppRoutes() {
  const { isFirstRun, error, isLoading } = useWorkspace();

  if (isLoading || isFirstRun === null) {
    return (
      <div className="flex h-screen flex-col items-center justify-center gap-4 bg-background text-foreground">
        <p>Carregando...</p>
        {error && (
          <p className="max-w-md text-center text-sm text-destructive">
            {error}
          </p>
        )}
      </div>
    );
  }

  if (error && isFirstRun === null) {
    return (
      <div className="flex h-screen flex-col items-center justify-center gap-4 bg-background p-8 text-foreground">
        <p className="text-destructive">{error}</p>
        <p className="text-sm text-muted-foreground">
          Verifique o console ou o arquivo de log para mais detalhes.
        </p>
      </div>
    );
  }

  const router = createHashRouter([
    {
      path: "/",
      element: isFirstRun === true ? <Onboarding /> : <AppLayout />,
      children:
        isFirstRun === false
          ? [
              { index: true, element: <Dashboard /> },
              { path: "npc", element: <NPCPage /> },
              { path: "lore", element: <LorePage /> },
              { path: "vault", element: <VaultPage /> },
              { path: "settings", element: <SettingsPage /> },
              { path: "*", element: <Navigate to="/" replace /> },
            ]
          : [],
    },
  ]);

  return <RouterProvider router={router} />;
}

function App() {
  return (
    <WorkspaceProvider>
      <AppRoutes />
    </WorkspaceProvider>
  );
}

export default App;
