import { invoke } from "@tauri-apps/api/core";

export interface SearchHit {
  source: "message" | "task" | "session";
  sourceId: string;
  title: string;
  snippet: string;
  createdAt: string;
  conversationId: string | null;
}

export const search = {
  query: (query: string, limit?: number) =>
    invoke<SearchHit[]>("search_index", { query, limit: limit ?? null }),
};
