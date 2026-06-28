import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useWorkspace } from "@/contexts/WorkspaceContext";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import { Send, Loader2, Bot, User } from "lucide-react";

interface ChatMessage {
  role: string;
  content: string;
  timestamp: number;
}

interface ChatResponse {
  reply: string;
  conversation_id: string;
  messages: ChatMessage[];
  error?: string;
}

export function ChatPage() {
  const { workspace } = useWorkspace();
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [conversationId, setConversationId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, loading]);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!input.trim() || !workspace?.rootPath) return;

    const userMessage = input.trim();
    setInput("");
    setLoading(true);
    setError(null);

    try {
      const response = await invoke<ChatResponse>("chat_with_hermes_command", {
        root: workspace.rootPath,
        request: {
          prompt: userMessage,
          history: messages,
          conversation_id: conversationId,
          module: null,
        },
      });

      if (response.error) {
        setError(response.error);
      } else {
        setConversationId(response.conversation_id);
        setMessages(response.messages);
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  if (!workspace?.rootPath) {
    return (
      <div className="flex h-full flex-col items-center justify-center text-center">
        <Bot className="h-12 w-12 text-muted-foreground" />
        <h2 className="mt-4 text-xl font-semibold">Nenhum projeto aberto</h2>
        <p className="text-sm text-muted-foreground">
          Abra ou crie um projeto para começar a conversar com o Hermes.
        </p>
      </div>
    );
  }

  return (
    <div className="flex h-[calc(100vh-7rem)] flex-col gap-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Chat com Hermes</h2>
          <p className="text-sm text-muted-foreground">
            Assistente de IA local via Hermes CLI.
          </p>
        </div>
      </div>

      {error && (
        <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
          {error}
        </div>
      )}

      <Card className="flex flex-1 flex-col overflow-hidden">
        <CardContent className="flex flex-1 flex-col overflow-hidden p-0">
          <div className="flex-1 space-y-4 overflow-y-auto p-4">
            {messages.length === 0 && (
              <div className="flex h-full flex-col items-center justify-center text-center text-muted-foreground">
                <Bot className="h-10 w-10" />
                <p className="mt-2">Comece digitando uma mensagem abaixo.</p>
              </div>
            )}
            {messages.map((msg, idx) => (
              <div
                key={idx}
                className={`flex gap-3 ${
                  msg.role === "user" ? "justify-end" : "justify-start"
                }`}
              >
                {msg.role !== "user" && (
                  <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-primary/10">
                    <Bot className="h-4 w-4" />
                  </div>
                )}
                <div
                  className={`max-w-[80%] rounded-lg px-4 py-2 text-sm ${
                    msg.role === "user"
                      ? "bg-primary text-primary-foreground"
                      : "bg-muted"
                  }`}
                >
                  {msg.content}
                </div>
                {msg.role === "user" && (
                  <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-muted">
                    <User className="h-4 w-4" />
                  </div>
                )}
              </div>
            ))}
            {loading && (
              <div className="flex items-center gap-3 text-muted-foreground">
                <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10">
                  <Bot className="h-4 w-4" />
                </div>
                <div className="flex items-center gap-2 rounded-lg bg-muted px-4 py-2 text-sm">
                  <Loader2 className="h-3 w-3 animate-spin" />
                  Hermes está pensando...
                </div>
              </div>
            )}
            <div ref={bottomRef} />
          </div>

          <form
            onSubmit={handleSubmit}
            className="flex items-center gap-2 border-t p-4"
          >
            <Input
              value={input}
              onChange={(e) => setInput(e.target.value)}
              placeholder="Digite sua mensagem..."
              disabled={loading}
              className="flex-1"
            />
            <Button type="submit" size="icon" disabled={!input.trim() || loading}>
              {loading ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Send className="h-4 w-4" />
              )}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
