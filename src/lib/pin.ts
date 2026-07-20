import { invoke } from "@tauri-apps/api/core";

export interface Pin {
  id: string;
  conversationId: string | null;
  messageId: string | null;
  label: string;
  content: string;
  x: number | null;
  y: number | null;
  createdAt: string;
}

interface CreatePinInput {
  conversationId?: string | null;
  messageId?: string | null;
  label?: string | null;
  content: string;
}

export const pins = {
  list: () => invoke<Pin[]>("list_pins"),
  get: (id: string) => invoke<Pin>("get_pin", { id }),
  create: (input: CreatePinInput) =>
    invoke<Pin>("create_pin", {
      conversationId: input.conversationId ?? null,
      messageId: input.messageId ?? null,
      label: input.label ?? null,
      content: input.content,
    }),
  updatePosition: (id: string, x: number, y: number) =>
    invoke<void>("update_pin_position", { id, x, y }),
  delete: (id: string) => invoke<void>("delete_pin", { id }),
  openWindow: (id: string, x?: number | null, y?: number | null) =>
    invoke<void>("open_pin_window", { pinId: id, x: x ?? null, y: y ?? null }),
  closeWindow: (id: string) => invoke<void>("close_pin_window", { pinId: id }),
};
