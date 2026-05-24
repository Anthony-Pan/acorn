import { formatDistanceToNowStrict } from "date-fns";
import {
  ArrowLeft,
  Loader2,
  MessageSquare,
  Plus,
  Send,
  Sparkles,
  Trash2,
  Wrench,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { AcornLogo } from "@/components/acorn-logo";
import { Button } from "@/components/ui/button";
import { VoiceButton } from "@/components/voice-button";
import { conversations as conversationsApi } from "@/lib/chat";
import { cn } from "@/lib/utils";
import { useChatStore } from "@/stores/chat";
import { useSettingsStore } from "@/stores/settings";
import type { Conversation, Message } from "@/types/chat";

interface ChatViewProps {
  onBack: () => void;
}

export function ChatView({ onBack }: ChatViewProps) {
  const current = useChatStore((s) => s.current);
  const conversations = useChatStore((s) => s.conversations);
  const phase = useChatStore((s) => s.phase);
  const activeTools = useChatStore((s) => s.activeTools);
  const error = useChatStore((s) => s.error);
  const send = useChatStore((s) => s.send);
  const startNew = useChatStore((s) => s.startNew);
  const open = useChatStore((s) => s.open);
  const hydrate = useChatStore((s) => s.hydrate);
  const rename = useChatStore((s) => s.rename);
  const activeProviderId = useSettingsStore((s) => s.activeProviderId);
  const [titleDraft, setTitleDraft] = useState<string | null>(null);
  const titleInputRef = useRef<HTMLInputElement>(null);

  const [draft, setDraft] = useState("");
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!current && conversations.length === 0) {
      void startNew();
    } else if (!current && conversations[0]) {
      void open(conversations[0].id);
    }
  }, [current, conversations, startNew, open]);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight, behavior: "smooth" });
  }, [current?.messages.length, activeTools.length, phase]);

  useEffect(() => {
    if (titleDraft !== null) {
      titleInputRef.current?.focus();
      titleInputRef.current?.select();
    }
  }, [titleDraft]);

  const handleDeleteConversation = async (conversationId: string) => {
    await conversationsApi.delete(conversationId);
    await hydrate();
    if (current?.id === conversationId) {
      await startNew();
    }
  };

  const canSend = draft.trim().length > 0 && phase === "idle" && !!activeProviderId;

  const handleSend = async () => {
    if (!canSend || !activeProviderId) return;
    const text = draft.trim();
    setDraft("");
    await send(text, activeProviderId);
  };

  return (
    <div className="min-h-screen bg-background flex">
      <ConversationSidebar
        conversations={conversations}
        activeId={current?.id ?? null}
        onSelect={(id) => void open(id)}
        onNew={() => void startNew()}
        onDelete={(id) => void handleDeleteConversation(id)}
        onBack={onBack}
      />

      <div className="flex-1 flex flex-col min-w-0">
        <header className="flex items-center justify-between px-7 py-4 border-b-[0.5px] border-border">
          <div className="flex items-center gap-2.5 min-w-0">
            <AcornLogo size={22} />
            {titleDraft !== null && current ? (
              <input
                ref={titleInputRef}
                value={titleDraft}
                onChange={(e) => setTitleDraft(e.target.value)}
                onBlur={async () => {
                  const next = titleDraft.trim();
                  setTitleDraft(null);
                  if (next && next !== current.title) {
                    await rename(next);
                  }
                }}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    (e.currentTarget as HTMLInputElement).blur();
                  }
                  if (e.key === "Escape") {
                    setTitleDraft(null);
                  }
                }}
                className="text-sm font-medium text-foreground bg-transparent border-b border-acorn-orange/40 focus:outline-none focus:border-acorn-orange px-0.5"
              />
            ) : (
              <button
                type="button"
                onDoubleClick={() => {
                  if (current) setTitleDraft(current.title);
                }}
                title="Double-click to rename"
                className="text-sm font-medium text-foreground truncate hover:text-acorn-orange transition-colors"
              >
                {current?.title ?? "Chatting with Acorn"}
              </button>
            )}
          </div>
        </header>

        <div ref={scrollRef} className="flex-1 overflow-y-auto px-7 py-5">
          <div className="max-w-2xl mx-auto space-y-4">
            {current?.messages
              .filter(
                (m) => m.role !== "tool" && (m.content.trim().length > 0 || m.role === "assistant"),
              )
              .map((message) => (
                <Bubble key={message.id} message={message} />
              ))}

            {activeTools.length > 0
              ? activeTools.map((tool) => (
                  <div
                    key={tool.id}
                    className="inline-flex items-center gap-2 text-xs text-muted-foreground bg-muted/60 rounded-full px-3 py-1.5"
                  >
                    <Wrench className="w-3 h-3" />
                    <span className="font-mono">{tool.name}</span>
                    {tool.result ? (
                      <span className="text-acorn-olive">· done</span>
                    ) : (
                      <Loader2 className="w-3 h-3 animate-spin" />
                    )}
                  </div>
                ))
              : null}

            {phase === "thinking" ? (
              <div className="inline-flex items-center gap-2 text-xs text-muted-foreground">
                <Sparkles className="w-3 h-3 text-acorn-orange animate-pulse" />
                Acorn is thinking...
              </div>
            ) : null}

            {error ? <div className="text-xs text-acorn-red">{error}</div> : null}
          </div>
        </div>

        <footer className="border-t-[0.5px] border-border px-7 py-4">
          <div className="max-w-2xl mx-auto">
            {!activeProviderId ? (
              <div className="text-xs text-muted-foreground mb-2">
                Configure a provider in Settings before you can chat.
              </div>
            ) : null}
            <div className="flex items-end gap-2">
              <textarea
                value={draft}
                onChange={(e) => setDraft(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && !e.shiftKey) {
                    e.preventDefault();
                    void handleSend();
                  }
                }}
                placeholder="Talk to Acorn..."
                rows={2}
                disabled={phase !== "idle"}
                className="flex-1 resize-none bg-card border-[0.5px] border-border rounded-xl px-4 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-ring/30 disabled:opacity-60"
              />
              <div className="flex flex-col gap-2">
                <VoiceButton
                  onTranscript={(text) => setDraft((d) => (d ? `${d} ${text}` : text))}
                />
                <Button onClick={() => void handleSend()} disabled={!canSend} size="sm">
                  {phase === "idle" ? (
                    <Send className="w-3.5 h-3.5" />
                  ) : (
                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  )}
                </Button>
              </div>
            </div>
          </div>
        </footer>
      </div>
    </div>
  );
}

interface ConversationSidebarProps {
  conversations: Conversation[];
  activeId: string | null;
  onSelect: (id: string) => void;
  onNew: () => void;
  onDelete: (id: string) => void;
  onBack: () => void;
}

function ConversationSidebar({
  conversations,
  activeId,
  onSelect,
  onNew,
  onDelete,
  onBack,
}: ConversationSidebarProps) {
  return (
    <aside className="w-60 flex-shrink-0 border-r-[0.5px] border-border flex flex-col">
      <div className="px-3 py-3 border-b-[0.5px] border-border flex items-center gap-2">
        <Button variant="outline" size="icon-sm" onClick={onBack} aria-label="Back">
          <ArrowLeft />
        </Button>
        <Button variant="outline" size="sm" onClick={onNew} className="flex-1 justify-center">
          <Plus />
          New chat
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto py-2">
        {conversations.length === 0 ? (
          <div className="px-4 py-6 text-xs text-muted-foreground">
            No chats yet. Start one above.
          </div>
        ) : (
          conversations.map((conv) => (
            <ConversationRow
              key={conv.id}
              conversation={conv}
              active={conv.id === activeId}
              onSelect={() => onSelect(conv.id)}
              onDelete={() => onDelete(conv.id)}
            />
          ))
        )}
      </div>
    </aside>
  );
}

function ConversationRow({
  conversation,
  active,
  onSelect,
  onDelete,
}: {
  conversation: Conversation;
  active: boolean;
  onSelect: () => void;
  onDelete: () => void;
}) {
  return (
    <div
      className={cn(
        "group mx-1 my-0.5 px-1 py-0.5 rounded-md transition-colors flex items-start gap-1",
        active ? "bg-acorn-orange/12" : "hover:bg-muted/60",
      )}
    >
      <button
        type="button"
        onClick={onSelect}
        className="flex-1 min-w-0 flex items-start gap-2 px-2 py-1.5 text-left"
      >
        <MessageSquare className="w-3.5 h-3.5 mt-0.5 text-muted-foreground flex-shrink-0" />
        <div className="flex-1 min-w-0">
          <div className="text-sm text-foreground truncate">{conversation.title}</div>
          <div className="text-[10px] text-muted-foreground">
            {formatDistanceToNowStrict(new Date(conversation.lastMessageAt), { addSuffix: true })}
          </div>
        </div>
      </button>
      <button
        type="button"
        onClick={onDelete}
        className="opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-acorn-red px-1 py-1"
        aria-label="Delete conversation"
      >
        <Trash2 className="w-3.5 h-3.5" />
      </button>
    </div>
  );
}

function Bubble({ message }: { message: Message }) {
  if (message.role === "user") {
    return (
      <div className="flex justify-end">
        <div className="max-w-[80%] bg-card border-[0.5px] border-border rounded-2xl rounded-br-md px-4 py-2.5 text-sm leading-relaxed">
          {message.content}
        </div>
      </div>
    );
  }

  if (message.role === "assistant") {
    const hasToolHint = message.toolCalls !== null;
    if (!message.content.trim() && !hasToolHint) return null;
    return (
      <div className="flex justify-start">
        <div className={cn("max-w-[80%] text-sm leading-relaxed prose prose-sm prose-stone")}>
          <ReactMarkdown remarkPlugins={[remarkGfm]}>{message.content}</ReactMarkdown>
        </div>
      </div>
    );
  }

  return null;
}
