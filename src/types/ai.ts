import type { Priority } from "./db";

export type ProviderCategory = "recommended" | "advanced" | "local" | "coming_soon";

export type ProviderStatus = "available" | "coming_soon";

export type ApiFormat = "anthropic" | "openai_compatible" | "ollama" | "gemini" | "acorn_cloud";

export interface ProviderMetadata {
  id: string;
  displayName: string;
  category: ProviderCategory;
  apiFormat: ApiFormat;
  defaultModel: string;
  availableModels: string[];
  defaultEndpoint: string;
  allowCustomEndpoint: boolean;
  requiresApiKey: boolean;
  apiKeyUrl: string | null;
  featured: boolean;
  status: ProviderStatus;
}

export interface DecomposeRequest {
  rawInput: string;
  language: string;
  userContext?: string | null;
}

export interface DecomposedSubtask {
  title: string;
  done: boolean;
}

export interface DecomposedTask {
  id: string;
  title: string;
  description: string | null;
  durationMinutes: number;
  priority: Priority;
  order: number;
  subtasks: DecomposedSubtask[];
}

export interface DecomposeResponse {
  tasks: DecomposedTask[];
  summary: string;
}

export type DecomposeEvent =
  | { kind: "progress"; receivedChars: number }
  | { kind: "task"; task: DecomposedTask }
  | { kind: "summary"; summary: string }
  | { kind: "done" };
