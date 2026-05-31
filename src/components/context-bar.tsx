import { cn } from "@/lib/utils";

interface ContextBarProps {
  contents: string[];
  draft: string;
}

// A pragmatic default context window for modern chat models. Token counts are
// estimated (~4 chars/token), so this is a "how full is the conversation"
// indicator rather than an exact meter.
const BUDGET_TOKENS = 128_000;
const MIN_TOKENS_TO_SHOW = 400;

function formatTokens(n: number): string {
  return n >= 1000 ? `${(n / 1000).toFixed(1)}k` : String(n);
}

/**
 * A slim progress bar showing roughly how much of the model's context window the
 * current conversation occupies, so it's clear at a glance how much room is left.
 */
export function ContextBar({ contents, draft }: ContextBarProps) {
  const chars = contents.reduce((sum, c) => sum + c.length, 0) + draft.length;
  const tokens = Math.ceil(chars / 4);
  if (tokens < MIN_TOKENS_TO_SHOW) return null;

  const pct = Math.min(100, (tokens / BUDGET_TOKENS) * 100);
  const near = pct > 80;

  return (
    <div className="mb-2">
      <div className="mb-1 flex items-center justify-between text-[10px] text-muted-foreground">
        <span>Context</span>
        <span className={cn("tabular-nums", near && "text-acorn-red")}>
          ~{formatTokens(tokens)} / {formatTokens(BUDGET_TOKENS)}
        </span>
      </div>
      <div className="h-1 overflow-hidden rounded-full bg-muted">
        <div
          className={cn(
            "h-full rounded-full transition-[width] duration-300",
            near ? "bg-acorn-red" : "bg-acorn-orange",
          )}
          style={{ width: `${Math.max(2, pct)}%` }}
        />
      </div>
    </div>
  );
}
