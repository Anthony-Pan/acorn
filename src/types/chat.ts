export type MessageRole = "user" | "assistant" | "tool" | "system";

export interface Conversation {
  id: string;
  title: string;
  providerId: string | null;
  model: string | null;
  createdAt: string;
  lastMessageAt: string;
}

export interface ToolCall {
  id: string;
  name: string;
  arguments: Record<string, unknown>;
}

export interface Message {
  id: string;
  conversationId: string;
  role: MessageRole;
  content: string;
  toolCalls: string | null;
  toolCallId: string | null;
  createdAt: string;
}

export interface ConversationWithMessages extends Conversation {
  messages: Message[];
}

/** A conversation summarized for the knowledge-cloud starfield. */
export interface CloudNode {
  id: string;
  title: string;
  lastMessageAt: string;
  messageCount: number;
  favorite: boolean;
  archived: boolean;
}

export type ChatEvent =
  | { kind: "thinking" }
  | { kind: "toolCall"; call: ToolCall }
  | { kind: "toolResult"; result: { toolCallId: string; content: string } }
  | { kind: "toolApprovalRequest"; requestId: string; call: ToolCall }
  | { kind: "toolDenied"; call: ToolCall }
  | { kind: "text"; content: string }
  | { kind: "done" }
  | { kind: "error"; message: string };
