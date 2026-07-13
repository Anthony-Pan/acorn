import { create } from "zustand";

import { decompose } from "@/lib/ai";
import { sessions, tasks } from "@/lib/db";
import type { DecomposedTask } from "@/types/ai";
import type { SessionWithTasks, Task, TaskStatus } from "@/types/db";

interface SessionState {
  current: SessionWithTasks | null;
  isStashing: boolean;
  progress: number;
  error: string | null;

  hydrate: () => Promise<void>;
  stash: (rawInput: string, providerId: string, language: string) => Promise<void>;
  updateTaskStatus: (taskId: string, status: TaskStatus) => Promise<void>;
  startNew: () => void;
}

function previewTask(decomposed: DecomposedTask, sessionId: string): Task {
  return {
    id: `preview-${decomposed.id}`,
    sessionId,
    title: decomposed.title,
    description: decomposed.description,
    durationMinutes: decomposed.durationMinutes,
    priority: decomposed.priority,
    orderIndex: decomposed.order,
    status: "pending",
    createdAt: new Date().toISOString(),
    startedAt: null,
    completedAt: null,
    scheduledStart: null,
    scheduledEnd: null,
    dueDate: null,
    allDay: false,
    updatedAt: new Date().toISOString(),
  };
}

export const useSessionStore = create<SessionState>((set) => ({
  current: null,
  isStashing: false,
  progress: 0,
  error: null,

  hydrate: async () => {
    const todays = await sessions.listToday();
    set({ current: todays[0] ?? null });
  },

  stash: async (rawInput, providerId, language) => {
    set({ isStashing: true, progress: 0, error: null });
    try {
      const session = await sessions.create(rawInput, language);
      set({ current: { ...session, tasks: [] } });

      const response = await decompose(providerId, { rawInput, language }, (event) => {
        if (event.kind === "progress") {
          set({ progress: event.receivedChars });
          return;
        }
        if (event.kind === "task") {
          set((s) =>
            s.current
              ? {
                  current: {
                    ...s.current,
                    tasks: [...s.current.tasks, previewTask(event.task, session.id)],
                  },
                }
              : s,
          );
          return;
        }
        if (event.kind === "summary") {
          set((s) => (s.current ? { current: { ...s.current, aiSummary: event.summary } } : s));
        }
      });

      const persisted = await Promise.all(
        response.tasks.map((t) =>
          tasks.insert({
            sessionId: session.id,
            title: t.title,
            description: t.description,
            durationMinutes: t.durationMinutes,
            priority: t.priority,
            orderIndex: t.order,
            subtasks: t.subtasks,
          }),
        ),
      );

      const finalized = await sessions.finalize(session.id, response.summary, providerId, "");
      set({
        current: { ...finalized, tasks: persisted },
        isStashing: false,
        progress: 0,
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      set({ isStashing: false, error: message });
    }
  },

  updateTaskStatus: async (taskId, status) => {
    const updated = await tasks.updateStatus(taskId, status);
    set((s) =>
      s.current
        ? {
            current: {
              ...s.current,
              tasks: s.current.tasks.map((t) => (t.id === taskId ? updated : t)),
            },
          }
        : s,
    );
  },

  startNew: () => {
    set({ current: null, progress: 0, error: null });
  },
}));
