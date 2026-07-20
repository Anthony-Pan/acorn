import { Copy, StickyNote } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";

interface InlineNote {
  title: string | null;
  body: string;
}

interface NoteCardsProps {
  raw: string;
}

function parseNotes(raw: string): InlineNote[] {
  const trimmed = raw.trim();
  if (!trimmed) return [];
  try {
    const parsed: unknown = JSON.parse(trimmed);
    if (Array.isArray(parsed)) {
      return parsed
        .filter(
          (item): item is Record<string, unknown> => typeof item === "object" && item !== null,
        )
        .map((item) => {
          const body = typeof item.body === "string" ? item.body.trim() : "";
          if (!body) return null;
          const title = typeof item.title === "string" ? item.title.trim() : "";
          return { title: title || null, body };
        })
        .filter((n): n is InlineNote => n !== null);
    }
  } catch {}
  return trimmed
    .split(/\n{2,}/)
    .map((chunk) => chunk.trim())
    .filter((chunk) => chunk.length > 0)
    .map((chunk) => ({ title: null, body: chunk }));
}

export function NoteCards({ raw }: NoteCardsProps) {
  const notes = useMemo(() => parseNotes(raw), [raw]);
  const [dismissed, setDismissed] = useState<Set<number>>(() => new Set());

  if (notes.length === 0) {
    return (
      <pre className="text-[11px] font-mono bg-muted/40 rounded-md p-3 overflow-x-auto">
        <code>{raw}</code>
      </pre>
    );
  }

  const visible = notes
    .map((note, idx) => ({ note, idx }))
    .filter(({ idx }) => !dismissed.has(idx));

  if (visible.length === 0) {
    return <div className="text-[11px] text-muted-foreground italic">All notes dismissed.</div>;
  }

  return (
    <div className="not-prose grid gap-2 my-3">
      {visible.map(({ note, idx }) => (
        <article
          key={idx}
          className="border-[0.5px] border-acorn-orange/25 rounded-lg px-3 py-2.5 bg-acorn-orange/5 flex items-start gap-3"
        >
          <StickyNote className="w-4 h-4 text-acorn-orange flex-shrink-0 mt-0.5" />
          <div className="flex-1 min-w-0">
            {note.title ? (
              <div className="text-[13px] font-medium text-foreground mb-0.5">{note.title}</div>
            ) : null}
            <div className="text-[13px] text-foreground/85 whitespace-pre-wrap break-words">
              {note.body}
            </div>
          </div>
          <div className="flex flex-col gap-1 flex-shrink-0">
            <Button
              variant="outline"
              size="icon-sm"
              aria-label="Copy note"
              onClick={async () => {
                try {
                  const text = note.title ? `${note.title}\n\n${note.body}` : note.body;
                  await navigator.clipboard.writeText(text);
                  toast.success("Note copied");
                } catch (err) {
                  toast.error("Could not copy", {
                    description: err instanceof Error ? err.message : String(err),
                  });
                }
              }}
            >
              <Copy className="w-3.5 h-3.5" />
            </Button>
            <Button
              variant="outline"
              size="icon-sm"
              aria-label="Dismiss note"
              onClick={() =>
                setDismissed((prev) => {
                  const next = new Set(prev);
                  next.add(idx);
                  return next;
                })
              }
            >
              ✕
            </Button>
          </div>
        </article>
      ))}
    </div>
  );
}
