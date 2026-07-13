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
  /** Calendar-event mapping: start of the timed block (RFC3339); null = undated. */
  scheduledStart: string | null;
  /** Calendar-event mapping: end of the block; null derives from durationMinutes. */
  scheduledEnd: string | null;
  /** Reminder/task mapping: due date (Google Tasks honours the date only). */
  dueDate: string | null;
  /** All-day vs timed for event mapping. */
  allDay: boolean;
  /** Last-write-wins clock; bumped by every mutating task command. */
  updatedAt: string;
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
