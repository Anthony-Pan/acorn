import { invoke } from "@tauri-apps/api/core";

import type {
  NewTaskInput,
  ProviderConfig,
  Session,
  SessionWithTasks,
  Subtask,
  Task,
  TaskStatus,
} from "@/types/db";

export const sessions = {
  create: (rawInput: string, language: string) =>
    invoke<Session>("create_session", { rawInput, language }),

  finalize: (sessionId: string, aiSummary: string, providerId: string, model: string) =>
    invoke<Session>("finalize_session", { sessionId, aiSummary, providerId, model }),

  get: (sessionId: string) => invoke<SessionWithTasks>("get_session", { sessionId }),

  listToday: () => invoke<SessionWithTasks[]>("list_today_sessions"),

  listRecent: (limit = 20) => invoke<Session[]>("list_recent_sessions", { limit }),
};

export const tasks = {
  insert: (input: NewTaskInput) => invoke<Task>("insert_task", { input }),

  updateStatus: (taskId: string, status: TaskStatus) =>
    invoke<Task>("update_task_status", { taskId, status }),

  listSubtasks: (taskId: string) => invoke<Subtask[]>("list_subtasks", { taskId }),

  toggleSubtask: (subtaskId: string) => invoke<Subtask>("toggle_subtask", { subtaskId }),

  setSchedule: (
    taskId: string,
    scheduledStart: string | null,
    scheduledEnd: string | null,
    dueDate: string | null,
    allDay: boolean,
  ) => invoke<Task>("set_task_schedule", { taskId, scheduledStart, scheduledEnd, dueDate, allDay }),

  remove: (taskId: string) => invoke<void>("delete_task", { taskId }),
};

export const settings = {
  get: (key: string) => invoke<string | null>("get_setting", { key }),

  set: (key: string, value: string) => invoke<void>("set_setting", { key, value }),

  remove: (key: string) => invoke<void>("delete_setting", { key }),
};

export const providers = {
  list: () => invoke<ProviderConfig[]>("list_provider_configs"),

  save: (
    providerId: string,
    enabled: boolean,
    customEndpoint: string | null,
    selectedModel: string | null,
  ) =>
    invoke<ProviderConfig>("save_provider_config", {
      providerId,
      enabled,
      customEndpoint,
      selectedModel,
    }),

  touch: (providerId: string) => invoke<void>("touch_provider", { providerId }),

  getActive: () => invoke<string | null>("get_active_provider"),

  setActive: (providerId: string) => invoke<void>("set_active_provider", { providerId }),
};
