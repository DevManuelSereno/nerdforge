import { Users } from "lucide-react";
import { Button } from "@/components/ui/button";

export function NPCPage() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold tracking-tight flex items-center gap-2">
          <Users className="h-6 w-6" />
          NPCs
        </h2>
        <p className="text-muted-foreground">
          Gerencie personagens não-jogadores do seu mundo.
        </p>
      </div>

      <div className="rounded-lg border border-dashed border-border p-8 text-center">
        <p className="text-muted-foreground">
          Módulo NPC será implementado no Slice 2.x.
        </p>
        <Button className="mt-4" disabled>
          Criar NPC
        </Button>
      </div>
    </div>
  );
}
