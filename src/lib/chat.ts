import { Channel, invoke } from "@tauri-apps/api/core";

import type {
  ChatEvent,
  CloudNode,
  Conversation,
  ConversationWithMessages,
  Message,
} from "@/types/chat";

export const conversations = {
  create: (title?: string) => invoke<Conversation>("create_conversation", { title: title ?? null }),

  list: () => invoke<Conversation[]>("list_conversations"),

  cloud: (includeArchived = false) =>
    invoke<CloudNode[]>("list_conversation_cloud", { includeArchived }),

  setFavorite: (conversationId: string, favorite: boolean) =>
    invoke<void>("set_conversation_favorite", { conversationId, favorite }),

  setArchived: (conversationId: string, archived: boolean) =>
    invoke<void>("set_conversation_archived", { conversationId, archived }),

  get: (conversationId: string) =>
    invoke<ConversationWithMessages>("get_conversation", { conversationId }),

  delete: (conversationId: string) => invoke<void>("delete_conversation", { conversationId }),

  rename: (conversationId: string, title: string) =>
    invoke<Conversation>("rename_conversation", { conversationId, title }),

  setProvider: (conversationId: string, providerId: string, model: string | null = null) =>
    invoke<Conversation>("set_conversation_provider", { conversationId, providerId, model }),
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
