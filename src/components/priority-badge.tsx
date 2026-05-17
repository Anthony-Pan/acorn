import { cn } from "@/lib/utils";
import type { Priority } from "@/types/db";

interface PriorityBadgeProps {
  priority: Priority;
  className?: string;
}

const STYLES: Record<Priority, string> = {
  high: "bg-acorn-orange text-acorn-paper",
  medium: "bg-acorn-orange/15 text-[var(--acorn-brown-deep)]",
  low: "bg-acorn-orange/10 text-muted-foreground",
};

const LABELS: Record<Priority, string> = {
  high: "HIGH",
  medium: "MED",
  low: "LOW",
};

export function PriorityBadge({ priority, className }: PriorityBadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center px-2 py-[2px] text-[10px] font-medium rounded-full leading-none tracking-wide",
        STYLES[priority],
        className,
      )}
    >
      {LABELS[priority]}
    </span>
  );
}
