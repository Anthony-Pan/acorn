import { toast } from "sonner";
import { create } from "zustand";

import { activity } from "@/lib/activity";
import { conversations as conversationsApi, chat as runChat } from "@/lib/chat";
import { useSessionStore } from "@/stores/session";
import type { ChatEvent, Conversation, ConversationWithMessages, Message } from "@/types/chat";

type Phase = "idle" | "thinking" | "tool" | "responding";

interface ActiveToolCall {
  id: string;
  name: string;
  result?: string;
}

interface ChatState {
  conversations: Conversation[];
  current: ConversationWithMessages | null;
  phase: Phase;
  activeTools: ActiveToolCall[];
  error: string | null;

  hydrate: () => Promise<void>;
  open: (conversationId: string) => Promise<void>;
  startNew: () => Promise<void>;
  rename: (title: string) => Promise<void>;
  send: (text: string, providerId: string) => Promise<void>;
}

export const useChatStore = create<ChatState>((set, get) => ({
  conversations: [],
  current: null,
  phase: "idle",
  activeTools: [],
  error: null,

  hydrate: async () => {
    const list = await conversationsApi.list();
    set({ conversations: list });
  },

  open: async (conversationId) => {
    const full = await conversationsApi.get(conversationId);
    set({ current: full, activeTools: [], phase: "idle", error: null });
  },

  startNew: async () => {
    const conv = await conversationsApi.create();
    set((s) => ({
      conversations: [conv, ...s.conversations],
      current: { ...conv, messages: [] },
      activeTools: [],
      phase: "idle",
      error: null,
    }));
  },

  rename: async (title) => {
    const current = get().current;
    if (!current) return;
    const renamed = await conversationsApi.rename(current.id, title);
    set((s) => ({
      current: s.current ? { ...s.current, title: renamed.title } : s.current,
      conversations: s.conversations.map((c) =>
        c.id === renamed.id ? { ...c, title: renamed.title } : c,
      ),
    }));
  },

  send: async (text, providerId) => {
    let conversation = get().current;
    if (!conversation) {
      const created = await conversationsApi.create();
      conversation = { ...created, messages: [] };
      set((s) => ({
        conversations: [created, ...s.conversations],
        current: conversation,
      }));
    }

    const optimisticUser: Message = {
      id: `pending-${crypto.randomUUID()}`,
      conversationId: conversation.id,
      role: "user",
      content: text,
      toolCalls: null,
      toolCallId: null,
      createdAt: new Date().toISOString(),
    };

    set((s) => ({
      current: s.current
        ? { ...s.current, messages: [...s.current.messages, optimisticUser] }
        : s.current,
      phase: "thinking",
      activeTools: [],
      error: null,
    }));

    void activity.record("chat:user-message", text).catch(() => {});

    const handleEvent = (event: ChatEvent) => {
      if (event.kind === "thinking") {
        set({ phase: "thinking" });
        return;
      }
      if (event.kind === "toolCall") {
        set((s) => ({
          phase: "tool",
          activeTools: [...s.activeTools, { id: event.call.id, name: event.call.name }],
        }));
        return;
      }
      if (event.kind === "toolResult") {
        const callId = event.result.toolCallId;
        const matching = get().activeTools.find((t) => t.id === callId);
        set((s) => ({
          activeTools: s.activeTools.map((t) =>
            t.id === callId ? { ...t, result: event.result.content } : t,
          ),
        }));
        if (matching) {
          announceToolResult(matching.name, event.result.content);
        }
        return;
      }
      if (event.kind === "text") {
        set({ phase: "responding" });
        return;
      }
      if (event.kind === "done") {
        set({ phase: "idle", activeTools: [] });
        return;
      }
      if (event.kind === "error") {
        set({ phase: "idle", error: event.message });
      }
    };

    try {
      await runChat(providerId, conversation.id, text, handleEvent);
      const refreshed = await conversationsApi.get(conversation.id);
      set({ current: refreshed, phase: "idle", activeTools: [] });
      await useSessionStore.getState().hydrate();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      set({ phase: "idle", error: message });
      toast.error("Chat failed", { description: message });
    }
  },
}));

function announceToolResult(toolName: string, resultJson: string): void {
  try {
    const parsed = JSON.parse(resultJson) as Record<string, unknown>;
    if (parsed.error) {
      toast.error(`${toolName} failed`, { description: String(parsed.error) });
      return;
    }
    if (toolName === "add_task" && typeof parsed.title === "string") {
      toast.success(`Added "${parsed.title}"`);
    } else if (toolName === "complete_task") {
      toast.success("Marked as done");
    } else if (toolName === "start_task") {
      toast.success("Started");
    } else if (toolName === "skip_task") {
      toast("Skipped");
    }
  } catch {
    return;
  }
}
