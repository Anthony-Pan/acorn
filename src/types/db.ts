export type Priority = "high" | "medium" | "low";

export type TaskStatus = "pending" | "in_progress" | "completed" | "skipped";

export interface Session {
  id: string;
  rawInput: string;
  aiSummary: string | null;
  language: string;
  providerId: string | null;
  model: string | null;
  createdAt: string;
  completedAt: string | null;
}

export interface Task {
  id: string;
  sessionId: string;
  title: string;
  description: string | null;
  durationMinutes: number;
  priority: Priority;
  orderIndex: number;
  status: TaskStatus;
  createdAt: string;
  startedAt: string | null;
  completedAt: string | null;
}

export interface Subtask {
  id: string;
  taskId: string;
  title: string;
  done: boolean;
  orderIndex: number;
}

export interface ProviderConfig {
  providerId: string;
  enabled: boolean;
  customEndpoint: string | null;
  selectedModel: string | null;
  lastUsedAt: string | null;
}

export interface SessionWithTasks extends Session {
  tasks: Task[];
}

export interface NewSubtaskInput {
  title: string;
}

export interface NewTaskInput {
  sessionId: string;
  title: string;
  description?: string | null;
  durationMinutes: number;
  priority: Priority;
  orderIndex: number;
  subtasks?: NewSubtaskInput[];
}
