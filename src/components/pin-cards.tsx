import { Pin, X } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";

interface InlinePin {
  label: string;
  value: string;
}

interface PinCardsProps {
  raw: string;
}

function parsePins(raw: string): InlinePin[] {
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
          const value = typeof item.value === "string" ? item.value.trim() : "";
          if (!value) return null;
          const labelRaw =
            typeof item.label === "string"
              ? item.label.trim()
              : typeof item.title === "string"
                ? item.title.trim()
                : "";
          return { label: labelRaw || "Pinned", value };
        })
        .filter((p): p is InlinePin => p !== null);
    }
  } catch {}
  return trimmed
    .split(/\n+/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
    .map((line) => {
      const colon = line.indexOf(":");
      if (colon > 0 && colon < 40) {
        return {
          label: line.slice(0, colon).trim(),
          value: line.slice(colon + 1).trim(),
        };
      }
      return { label: "Pinned", value: line };
    });
}

export function PinCards({ raw }: PinCardsProps) {
  const pins = useMemo(() => parsePins(raw), [raw]);
  const [dismissed, setDismissed] = useState<Set<number>>(() => new Set());

  if (pins.length === 0) {
    return (
      <pre className="text-xs font-mono bg-muted/40 rounded-md p-3 overflow-x-auto">
        <code>{raw}</code>
      </pre>
    );
  }

  const visible = pins.map((pin, idx) => ({ pin, idx })).filter(({ idx }) => !dismissed.has(idx));

  if (visible.length === 0) {
    return <div className="text-xs text-muted-foreground italic">All pins dismissed.</div>;
  }

  return (
    <div className="not-prose flex flex-wrap gap-2 my-3">
      {visible.map(({ pin, idx }) => (
        <div
          key={idx}
          className="group inline-flex items-center gap-2 border-[0.5px] border-acorn-brown/40 bg-acorn-brown/10 hover:bg-acorn-brown/15 rounded-full pl-3 pr-1 py-1 transition-colors"
        >
          <button
            type="button"
            onClick={async () => {
              try {
                await navigator.clipboard.writeText(pin.value);
                toast.success(`Copied: ${pin.label}`);
              } catch (err) {
                toast.error("Could not copy", {
                  description: err instanceof Error ? err.message : String(err),
                });
              }
            }}
            className="inline-flex items-center gap-2 focus:outline-none"
          >
            <Pin className="w-3 h-3 text-acorn-brown-deep" />
            <span className="text-[10px] uppercase tracking-wider text-acorn-brown-deep font-medium">
              {pin.label}
            </span>
            <span className="text-xs text-foreground/85 font-mono truncate max-w-[200px]">
              {pin.value}
            </span>
          </button>
          <button
            type="button"
            aria-label="Dismiss pin"
            onClick={() => {
              setDismissed((prev) => {
                const next = new Set(prev);
                next.add(idx);
                return next;
              });
            }}
            className="h-5 w-5 inline-flex items-center justify-center rounded hover:bg-acorn-brown/20 text-acorn-brown-deep"
          >
            <X className="w-3 h-3" />
          </button>
        </div>
      ))}
    </div>
  );
}
