import { createHashRouter, Navigate } from "react-router-dom";
import { AppLayout } from "@/layouts/AppLayout";
import { Dashboard } from "@/pages/Dashboard";
import { NPCPage } from "@/pages/NPCPage";
import { LorePage } from "@/pages/LorePage";
import { VaultPage } from "@/pages/VaultPage";
import { SettingsPage } from "@/pages/SettingsPage";

export const router = createHashRouter([
  {
    path: "/",
    element: <AppLayout />,
    children: [
      { index: true, element: <Dashboard /> },
      { path: "npc", element: <NPCPage /> },
      { path: "lore", element: <LorePage /> },
      { path: "vault", element: <VaultPage /> },
      { path: "settings", element: <SettingsPage /> },
      { path: "*", element: <Navigate to="/" replace /> },
    ],
  },
]);
