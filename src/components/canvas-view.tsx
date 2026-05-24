import { formatDistanceToNowStrict } from "date-fns";
import { ArrowLeft, FileText, Loader2, Plus, Trash2 } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { type Canvas, canvases } from "@/lib/canvas";
import { cn } from "@/lib/utils";

interface CanvasViewProps {
  onBack: () => void;
}

const AUTOSAVE_DEBOUNCE_MS = 800;

export function CanvasView({ onBack }: CanvasViewProps) {
  const [list, setList] = useState<Canvas[]>([]);
  const [current, setCurrent] = useState<Canvas | null>(null);
  const [draftTitle, setDraftTitle] = useState("");
  const [draftContent, setDraftContent] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [preview, setPreview] = useState(false);
  const saveTimer = useRef<number | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const next = await canvases.list();
      setList(next);
      if (!current && next[0]) {
        setCurrent(next[0]);
        setDraftTitle(next[0].title);
        setDraftContent(next[0].content);
      }
    } finally {
      setLoading(false);
    }
  }, [current]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    if (!current) return;
    if (draftTitle === current.title && draftContent === current.content) return;
    if (saveTimer.current !== null) window.clearTimeout(saveTimer.current);
    saveTimer.current = window.setTimeout(async () => {
      setSaving(true);
      try {
        const updated = await canvases.update(current.id, {
          title: draftTitle,
          content: draftContent,
        });
        setCurrent(updated);
        setList((prev) => prev.map((c) => (c.id === updated.id ? updated : c)));
      } finally {
        setSaving(false);
      }
    }, AUTOSAVE_DEBOUNCE_MS);
    return () => {
      if (saveTimer.current !== null) window.clearTimeout(saveTimer.current);
    };
  }, [draftTitle, draftContent, current]);

  const openCanvas = (canvas: Canvas) => {
    setCurrent(canvas);
    setDraftTitle(canvas.title);
    setDraftContent(canvas.content);
    setPreview(false);
  };

  const handleNew = async () => {
    const created = await canvases.create();
    setList((prev) => [created, ...prev]);
    openCanvas(created);
  };

  const handleDelete = async (canvas: Canvas) => {
    await canvases.delete(canvas.id);
    setList((prev) => prev.filter((c) => c.id !== canvas.id));
    if (current?.id === canvas.id) {
      const next = list.find((c) => c.id !== canvas.id) ?? null;
      if (next) openCanvas(next);
      else {
        setCurrent(null);
        setDraftTitle("");
        setDraftContent("");
      }
    }
    toast.success("Canvas deleted");
  };

  return (
    <div className="min-h-screen bg-background flex">
      <aside className="w-60 flex-shrink-0 border-r-[0.5px] border-border flex flex-col">
        <div className="px-3 py-3 border-b-[0.5px] border-border flex items-center gap-2">
          <Button variant="outline" size="icon-sm" onClick={onBack} aria-label="Back">
            <ArrowLeft />
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => void handleNew()}
            className="flex-1 justify-center"
          >
            <Plus />
            New canvas
          </Button>
        </div>
        <div className="flex-1 overflow-y-auto py-2">
          {loading ? (
            <div className="px-4 py-6 text-xs text-muted-foreground inline-flex items-center gap-2">
              <Loader2 className="w-3 h-3 animate-spin" />
              Loading…
            </div>
          ) : list.length === 0 ? (
            <div className="px-4 py-6 text-xs text-muted-foreground">No canvases yet.</div>
          ) : (
            list.map((c) => (
              <div
                key={c.id}
                className={cn(
                  "group mx-1 my-0.5 px-2 py-1.5 rounded-md flex items-start gap-2 transition-colors",
                  current?.id === c.id ? "bg-acorn-orange/12" : "hover:bg-muted/60",
                )}
              >
                <button
                  type="button"
                  onClick={() => openCanvas(c)}
                  className="flex-1 min-w-0 flex items-start gap-2 text-left"
                >
                  <FileText className="w-3.5 h-3.5 mt-0.5 text-muted-foreground flex-shrink-0" />
                  <div className="flex-1 min-w-0">
                    <div className="text-sm text-foreground truncate">{c.title}</div>
                    <div className="text-[10px] text-muted-foreground">
                      {formatDistanceToNowStrict(new Date(c.updatedAt), { addSuffix: true })}
                    </div>
                  </div>
                </button>
                <button
                  type="button"
                  aria-label="Delete canvas"
                  onClick={() => void handleDelete(c)}
                  className="opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-acorn-red px-1"
                >
                  <Trash2 className="w-3 h-3" />
                </button>
              </div>
            ))
          )}
        </div>
      </aside>

      <div className="flex-1 flex flex-col min-w-0">
        {current ? (
          <>
            <header className="flex items-center justify-between px-7 py-3.5 border-b-[0.5px] border-border">
              <input
                value={draftTitle}
                onChange={(e) => setDraftTitle(e.target.value)}
                placeholder="Canvas title"
                className="flex-1 text-sm font-medium text-foreground bg-transparent focus:outline-none placeholder:text-muted-foreground"
              />
              <div className="flex items-center gap-2">
                <span className="text-[10px] text-muted-foreground inline-flex items-center gap-1">
                  {saving ? <Loader2 className="w-3 h-3 animate-spin" /> : null}
                  {saving ? "Saving…" : "Saved"}
                </span>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setPreview((p) => !p)}
                  aria-pressed={preview}
                >
                  {preview ? "Edit" : "Preview"}
                </Button>
              </div>
            </header>
            <div className="flex-1 overflow-y-auto px-7 py-5">
              {preview ? (
                <div className="max-w-3xl mx-auto prose prose-sm prose-stone">
                  <ReactMarkdown remarkPlugins={[remarkGfm]}>
                    {draftContent || "_Empty canvas. Switch to Edit to start writing._"}
                  </ReactMarkdown>
                </div>
              ) : (
                <textarea
                  value={draftContent}
                  onChange={(e) => setDraftContent(e.target.value)}
                  placeholder="Markdown. Auto-saves while you type."
                  className="w-full max-w-3xl mx-auto min-h-[70vh] block bg-transparent text-sm font-mono leading-relaxed focus:outline-none resize-none"
                />
              )}
            </div>
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center text-sm text-muted-foreground">
            Pick a canvas, or start a new one.
          </div>
        )}
      </div>
    </div>
  );
}
