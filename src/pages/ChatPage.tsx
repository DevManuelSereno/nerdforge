import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useWorkspace } from "@/contexts/WorkspaceContext";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import { Send, Loader2, Bot, User, MessageSquarePlus, RefreshCw } from "lucide-react";

interface ChatMessage {
  role: string;
  content: string;
  timestamp: number;
}

interface Conversation {
  id: string;
  title: string | null;
  summary: string | null;
  context: string | null;
  created_at: number;
  updated_at: number;
  messages: ChatMessage[];
  module: string | null;
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
  const [conversationTitle, setConversationTitle] = useState<string | null>(null);
  const [conversationContext, setConversationContext] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [conversationsLoading, setConversationsLoading] = useState(false);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, loading]);

  useEffect(() => {
    if (workspace?.rootPath) {
      loadConversations();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [workspace?.rootPath]);

  async function loadConversations() {
    if (!workspace?.rootPath) return;
    setConversationsLoading(true);
    try {
      const result = await invoke<Conversation[]>("list_conversations_command", {
        root: workspace.rootPath,
      });
      setConversations(result);
    } catch (err) {
      console.error("Falha ao carregar conversas:", err);
    } finally {
      setConversationsLoading(false);
    }
  }

  async function loadConversation(id: string) {
    if (!workspace?.rootPath) return;
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<Conversation>("load_conversation_command", {
        root: workspace.rootPath,
        id,
      });
      setConversationId(result.id);
      setConversationTitle(result.title);
      setConversationContext(result.context);
      setMessages(result.messages);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  function startNewConversation() {
    setConversationId(null);
    setConversationTitle(null);
    setConversationContext(null);
    setMessages([]);
    setError(null);
  }

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
          context: conversationContext,
        },
      });

      if (response.error) {
        setError(response.error);
      } else {
        setConversationId(response.conversation_id);
        if (!conversationTitle) {
          setConversationTitle(userMessage.slice(0, 60));
        }
        setMessages(response.messages);
        await loadConversations();
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  async function handleSummarize() {
    if (!workspace?.rootPath || !conversationId) return;
    setLoading(true);
    setError(null);
    try {
      await invoke("summarize_conversation_command", {
        root: workspace.rootPath,
        conversation_id: conversationId,
      });
      await loadConversation(conversationId);
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
    <div className="flex h-[calc(100vh-7rem)] gap-4">
      {/* Sidebar de conversas */}
      <Card className="flex w-64 flex-col overflow-hidden">
        <CardContent className="flex flex-1 flex-col gap-2 overflow-hidden p-3">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-semibold">Conversas</h3>
            <div className="flex gap-1">
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7"
                onClick={loadConversations}
                disabled={conversationsLoading}
              >
                <RefreshCw
                  className={`h-3.5 w-3.5 ${conversationsLoading ? "animate-spin" : ""}`}
                />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7"
                onClick={startNewConversation}
                title="Nova conversa"
              >
                <MessageSquarePlus className="h-3.5 w-3.5" />
              </Button>
            </div>
          </div>

          <div className="flex-1 space-y-1 overflow-y-auto">
            {conversations.length === 0 && (
              <p className="text-xs text-muted-foreground">
                Nenhuma conversa ainda.
              </p>
            )}
            {conversations.map((conv) => (
              <button
                key={conv.id}
                onClick={() => loadConversation(conv.id)}
                className={`w-full rounded-md px-2 py-1.5 text-left text-xs transition-colors ${
                  conv.id === conversationId
                    ? "bg-primary text-primary-foreground"
                    : "hover:bg-muted"
                }`}
              >
                <p className="truncate font-medium">
                  {conv.title || "Conversa sem título"}
                </p>
                <p className="truncate text-[10px] opacity-80">
                  {conv.summary || `${conv.messages.length} mensagens`}
                </p>
              </button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Área de chat */}
      <div className="flex flex-1 flex-col gap-4">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold tracking-tight">Chat com Hermes</h2>
            <p className="text-sm text-muted-foreground">
              Assistente de IA local via Hermes CLI.
            </p>
          </div>
          {conversationId && (
            <Button
              variant="outline"
              size="sm"
              onClick={handleSummarize}
              disabled={loading || messages.length < 4}
            >
              Resumir conversa
            </Button>
          )}
        </div>

        {conversationContext && (
          <div className="rounded-md bg-muted px-3 py-2 text-xs text-muted-foreground">
            <span className="font-semibold">Contexto:</span> {conversationContext}
          </div>
        )}

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
    </div>
  );
}
