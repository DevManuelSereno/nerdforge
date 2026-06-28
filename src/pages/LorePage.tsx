import { BookOpen } from "lucide-react";
import { Button } from "@/components/ui/button";

export function LorePage() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold tracking-tight flex items-center gap-2">
          <BookOpen className="h-6 w-6" />
          Lore
        </h2>
        <p className="text-muted-foreground">
          Organize história, mitologia e conhecimento do mundo.
        </p>
      </div>

      <div className="rounded-lg border border-dashed border-border p-8 text-center">
        <p className="text-muted-foreground">
          Módulo Lore será implementado no Slice 3.x.
        </p>
        <Button className="mt-4" disabled>
          Criar entrada de Lore
        </Button>
      </div>
    </div>
  );
}
