import { Channel, invoke } from "@tauri-apps/api/core";

import type { ChatEvent, Conversation, ConversationWithMessages, Message } from "@/types/chat";

export const conversations = {
  create: (title?: string) => invoke<Conversation>("create_conversation", { title: title ?? null }),

  list: () => invoke<Conversation[]>("list_conversations"),

  get: (conversationId: string) =>
    invoke<ConversationWithMessages>("get_conversation", { conversationId }),

  delete: (conversationId: string) => invoke<void>("delete_conversation", { conversationId }),

  rename: (conversationId: string, title: string) =>
    invoke<Conversation>("rename_conversation", { conversationId, title }),
};

export function chat(
  providerId: string,
  conversationId: string,
  userMessage: string,
  onEvent: (event: ChatEvent) => void,
): Promise<Message> {
  const channel = new Channel<ChatEvent>();
  channel.onmessage = onEvent;
  return invoke<Message>("chat", {
    providerId,
    conversationId,
    userMessage,
    onEvent: channel,
  });
}
