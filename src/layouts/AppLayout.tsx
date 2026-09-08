import { Link, useLocation, Outlet } from "react-router-dom";
import { Home, FolderKanban, MessageSquare, Users, BookOpen, Settings, FolderOpen, Menu, X, Wand2, Swords, Map, AlertTriangle } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { useWorkspace } from "@/contexts/WorkspaceContext";

const navItems = [
  { path: "/", label: "Dashboard", icon: Home },
  { path: "/projects", label: "Projetos", icon: FolderKanban },
  { path: "/chat", label: "Chat", icon: MessageSquare },
  { path: "/structured-output", label: "Structured Output", icon: Wand2 },
  { path: "/npc", label: "NPCs", icon: Users },
  { path: "/lore", label: "Lore", icon: BookOpen },
  { path: "/factions", label: "Facções", icon: Swords },
  { path: "/locations", label: "Locais", icon: Map },
  { path: "/conflicts", label: "Conflitos", icon: AlertTriangle },
  { path: "/vault", label: "Vault", icon: FolderOpen },
  { path: "/settings", label: "Configurações", icon: Settings },
];

export function AppLayout() {
  const location = useLocation();
  const [mobileOpen, setMobileOpen] = useState(false);
  const { workspace, isLoading, error } = useWorkspace();

  return (
    <div className="flex h-screen w-full bg-background text-foreground">
      {/* Desktop sidebar */}
      <aside className="hidden w-64 flex-col border-r border-border bg-card md:flex">
        <div className="flex h-16 items-center border-b border-border px-6">
          <h1 className="text-xl font-bold tracking-tight">NerdForge</h1>
        </div>
        <nav className="flex-1 space-y-1 overflow-y-auto p-4">
          {navItems.map((item) => {
            const Icon = item.icon;
            const active = location.pathname === item.path;
            return (
              <Link
                key={item.path}
                to={item.path}
                className={`flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                  active
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:bg-muted hover:text-foreground"
                }`}
              >
                <Icon className="h-4 w-4" />
                {item.label}
              </Link>
            );
          })}
        </nav>
        <div className="border-t border-border p-4">
          <p className="truncate text-xs text-muted-foreground">
            {isLoading
              ? "Carregando workspace..."
              : workspace
                ? workspace.name
                : "Sem workspace"}
          </p>
          {error && (
            <p className="mt-1 text-xs text-destructive">{error}</p>
          )}
        </div>
      </aside>

      {/* Mobile sidebar overlay */}
      {mobileOpen && (
        <div
          className="fixed inset-0 z-40 bg-black/50 md:hidden"
          onClick={() => setMobileOpen(false)}
        />
      )}

      {/* Mobile sidebar drawer */}
      <aside
        className={`fixed inset-y-0 left-0 z-50 w-64 transform flex-col border-r border-border bg-card transition-transform duration-200 md:hidden ${mobileOpen ? "translate-x-0" : "-translate-x-full"}`}
      >
        <div className="flex h-16 items-center justify-between border-b border-border px-4">
          <h1 className="text-xl font-bold tracking-tight">NerdForge</h1>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setMobileOpen(false)}
          >
            <X className="h-5 w-5" />
          </Button>
        </div>
        <nav className="flex-1 space-y-1 overflow-y-auto p-4">
          {navItems.map((item) => {
            const Icon = item.icon;
            const active = location.pathname === item.path;
            return (
              <Link
                key={item.path}
                to={item.path}
                onClick={() => setMobileOpen(false)}
                className={`flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                  active
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:bg-muted hover:text-foreground"
                }`}
              >
                <Icon className="h-4 w-4" />
                {item.label}
              </Link>
            );
          })}
        </nav>
      </aside>

      {/* Main content */}
      <div className="flex flex-1 flex-col overflow-hidden">
        <header className="flex h-16 items-center justify-between border-b border-border bg-card px-4 md:px-6">
          <div className="flex items-center gap-3">
            <Button
              variant="ghost"
              size="icon"
              className="md:hidden"
              onClick={() => setMobileOpen(true)}
            >
              <Menu className="h-5 w-5" />
            </Button>
            <h2 className="text-lg font-semibold">
              {navItems.find((item) => item.path === location.pathname)
                ?.label || "NerdForge"}
            </h2>
          </div>
          <div className="flex items-center gap-2">
            {workspace && (
              <span className="hidden text-sm text-muted-foreground md:inline">
                {workspace.rootPath}
              </span>
            )}
          </div>
        </header>

        <main className="flex-1 overflow-auto p-4 md:p-6">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
