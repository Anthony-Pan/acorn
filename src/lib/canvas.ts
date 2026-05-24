import { invoke } from "@tauri-apps/api/core";

export interface Canvas {
  id: string;
  title: string;
  content: string;
  createdAt: string;
  updatedAt: string;
}

export const canvases = {
  list: () => invoke<Canvas[]>("list_canvases"),
  get: (id: string) => invoke<Canvas>("get_canvas", { id }),
  create: (title?: string) => invoke<Canvas>("create_canvas", { title: title ?? null }),
  update: (id: string, patch: { title?: string; content?: string }) =>
    invoke<Canvas>("update_canvas", {
      id,
      title: patch.title ?? null,
      content: patch.content ?? null,
    }),
  delete: (id: string) => invoke<void>("delete_canvas", { id }),
};
