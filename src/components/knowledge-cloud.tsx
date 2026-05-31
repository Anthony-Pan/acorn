import { formatDistanceToNowStrict } from "date-fns";
import { Archive, ArrowLeft, MessageSquare, Search, Star } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { conversations } from "@/lib/chat";
import { cn } from "@/lib/utils";
import type { CloudNode } from "@/types/chat";

interface KnowledgeCloudProps {
  onBack: () => void;
  onOpen: (conversationId: string) => void;
}

interface PlacedStar {
  node: CloudNode;
  cluster: string;
  x: number; // percent
  y: number; // percent
  size: number; // px
  brightness: number; // 0..1
}

const STOPWORDS = new Set([
  "the",
  "a",
  "an",
  "to",
  "of",
  "and",
  "or",
  "for",
  "with",
  "how",
  "what",
  "why",
  "when",
  "where",
  "my",
  "me",
  "i",
  "is",
  "are",
  "in",
  "on",
  "at",
  "it",
  "this",
  "that",
  "new",
  "untitled",
  "chat",
  "about",
  "can",
  "you",
  "please",
  "help",
  "need",
  "want",
  "let",
  "make",
  "get",
  "do",
  "does",
]);

// Deterministic 0..1 hash so a conversation always lands in the same spot.
function hashUnit(input: string): number {
  let h = 2166136261;
  for (let i = 0; i < input.length; i++) {
    h ^= input.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return (h >>> 0) / 4294967295;
}

// Pull a clustering keyword from a title. Latin titles cluster by their first
// meaningful word; scriptless titles (e.g. CJK) fall back to a leading bigram.
function keywordOf(title: string): string {
  const cleaned = title.toLowerCase().replace(/[^\p{L}\p{N}\s]/gu, " ");
  const words = cleaned.split(/\s+/).filter((w) => w.length > 1 && !STOPWORDS.has(w));
  if (words[0]) return words[0];
  const compact = title.replace(/\s+/g, "");
  return compact ? compact.slice(0, 2) : "misc";
}

function daysSince(iso: string): number {
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return 999;
  return (Date.now() - then) / 86_400_000;
}

/**
 * The knowledge cloud: every conversation rendered as a glowing star, gathered
 * into keyword constellations. Bigger/brighter = more and more-recent activity;
 * favorites glow gold. Hover lights up a constellation, click selects, and Open
 * jumps back into that conversation.
 */
export function KnowledgeCloud({ onBack, onOpen }: KnowledgeCloudProps) {
  const [nodes, setNodes] = useState<CloudNode[]>([]);
  const [loading, setLoading] = useState(true);
  const [includeArchived, setIncludeArchived] = useState(false);
  const [search, setSearch] = useState("");
  const [hoverCluster, setHoverCluster] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    void conversations
      .cloud(includeArchived)
      .then((list) => setNodes(list))
      .catch(() => setNodes([]))
      .finally(() => setLoading(false));
  }, [includeArchived]);

  const { stars, clusterCenters } = useMemo(() => {
    const clusters = new Map<string, CloudNode[]>();
    for (const node of nodes) {
      const key = keywordOf(node.title);
      const bucket = clusters.get(key);
      if (bucket) bucket.push(node);
      else clusters.set(key, [node]);
    }
    const clusterList = [...clusters.entries()];
    const centers = new Map<string, { x: number; y: number }>();
    const placed: PlacedStar[] = [];

    clusterList.forEach(([key, members], ci) => {
      // Spread cluster centers around the canvas on a ring (single cluster -> center).
      const angle = (ci / Math.max(1, clusterList.length)) * Math.PI * 2;
      const ringRadius = clusterList.length <= 1 ? 0 : 30;
      const cx = 50 + Math.cos(angle) * ringRadius;
      const cy = 50 + Math.sin(angle) * ringRadius * 0.78;
      centers.set(key, { x: cx, y: cy });

      members.forEach((node, ni) => {
        const seed = hashUnit(node.id);
        const memberAngle = seed * Math.PI * 2;
        const spread = members.length === 1 ? 0 : 6 + (ni % 5) * 2.6 + seed * 4;
        const x = Math.min(96, Math.max(4, cx + Math.cos(memberAngle) * spread));
        const y = Math.min(92, Math.max(8, cy + Math.sin(memberAngle) * spread * 0.82));
        const size = Math.min(30, 9 + Math.sqrt(node.messageCount) * 3);
        const recency = Math.max(0.35, 1 - daysSince(node.lastMessageAt) / 90);
        placed.push({ node, cluster: key, x, y, size, brightness: recency });
      });
    });

    return { stars: placed, clusterCenters: centers };
  }, [nodes]);

  const query = search.trim().toLowerCase();
  const matches = (star: PlacedStar) =>
    query.length === 0 || star.node.title.toLowerCase().includes(query);

  const selectedStar = stars.find((s) => s.node.id === selected) ?? null;

  const toggleFavorite = (node: CloudNode) => {
    const next = !node.favorite;
    setNodes((list) => list.map((n) => (n.id === node.id ? { ...n, favorite: next } : n)));
    void conversations.setFavorite(node.id, next).catch(() => {});
  };

  const toggleArchived = (node: CloudNode) => {
    const next = !node.archived;
    void conversations.setArchived(node.id, next).catch(() => {});
    setNodes((list) =>
      includeArchived
        ? list.map((n) => (n.id === node.id ? { ...n, archived: next } : n))
        : list.filter((n) => n.id !== node.id),
    );
    if (selected === node.id) setSelected(null);
  };

  return (
    <div className="relative h-screen w-screen overflow-hidden bg-gradient-to-b from-[#1a120b] via-[#241712] to-[#120b07] text-acorn-paper">
      {/* Constellation lines for the active cluster. */}
      <svg
        className="pointer-events-none absolute inset-0 h-full w-full"
        viewBox="0 0 100 100"
        preserveAspectRatio="none"
        aria-hidden="true"
      >
        <title>Constellation links</title>
        {stars
          .filter((s) => s.cluster === (hoverCluster ?? selectedStar?.cluster))
          .map((s) => {
            const center = clusterCenters.get(s.cluster);
            if (!center) return null;
            return (
              <line
                key={`line-${s.node.id}`}
                x1={center.x}
                y1={center.y}
                x2={s.x}
                y2={s.y}
                stroke="rgba(181,128,63,0.35)"
                strokeWidth={0.15}
              />
            );
          })}
      </svg>

      {/* Cluster labels. */}
      {[...clusterCenters.entries()].map(([key, center]) => (
        <span
          key={`label-${key}`}
          className={cn(
            "pointer-events-none absolute -translate-x-1/2 select-none text-[10px] uppercase tracking-[0.2em] transition-opacity",
            hoverCluster === key
              ? "text-acorn-orange opacity-100"
              : "text-acorn-paper/30 opacity-70",
          )}
          style={{ left: `${center.x}%`, top: `${center.y - 7}%` }}
        >
          {key}
        </span>
      ))}

      {/* Stars. */}
      {stars.map((star) => {
        const dim = !matches(star) || (hoverCluster !== null && hoverCluster !== star.cluster);
        const fav = star.node.favorite;
        return (
          <button
            key={star.node.id}
            type="button"
            onPointerEnter={() => setHoverCluster(star.cluster)}
            onPointerLeave={() => setHoverCluster(null)}
            onClick={() => setSelected(star.node.id)}
            onDoubleClick={() => onOpen(star.node.id)}
            title={star.node.title}
            className="absolute -translate-x-1/2 -translate-y-1/2 rounded-full transition-all duration-300 focus:outline-none"
            style={{
              left: `${star.x}%`,
              top: `${star.y}%`,
              width: star.size,
              height: star.size,
              opacity: dim ? 0.18 : star.brightness,
              background: fav
                ? "radial-gradient(circle at 35% 30%, #f4c971, #b5803f)"
                : "radial-gradient(circle at 35% 30%, #f0a868, #8b4513)",
              boxShadow: fav
                ? "0 0 14px 3px rgba(244,201,113,0.55)"
                : `0 0 ${selected === star.node.id ? 16 : 8}px ${selected === star.node.id ? 4 : 1}px rgba(240,168,104,0.5)`,
              transform: `translate(-50%, -50%) scale(${selected === star.node.id ? 1.35 : 1})`,
            }}
          />
        );
      })}

      {/* Top bar. */}
      <header className="absolute inset-x-0 top-0 flex items-center gap-3 px-6 py-4">
        <button
          type="button"
          onClick={onBack}
          className="flex items-center gap-1.5 rounded-full bg-white/5 px-3 py-1.5 text-xs text-acorn-paper/80 backdrop-blur transition-colors hover:bg-white/10"
        >
          <ArrowLeft className="h-3.5 w-3.5" />
          Back
        </button>
        <h1 className="text-sm font-medium tracking-wide text-acorn-paper/90">Knowledge Cloud</h1>
        <div className="ml-auto flex items-center gap-2">
          <div className="flex items-center gap-1.5 rounded-full bg-white/5 px-3 py-1.5 backdrop-blur">
            <Search className="h-3.5 w-3.5 text-acorn-paper/50" />
            <input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Search the cloud…"
              className="w-44 bg-transparent text-xs text-acorn-paper placeholder:text-acorn-paper/40 focus:outline-none"
            />
          </div>
          <button
            type="button"
            onClick={() => setIncludeArchived((v) => !v)}
            className={cn(
              "rounded-full px-3 py-1.5 text-xs backdrop-blur transition-colors",
              includeArchived
                ? "bg-acorn-orange/30 text-acorn-paper"
                : "bg-white/5 text-acorn-paper/70 hover:bg-white/10",
            )}
          >
            Archived
          </button>
        </div>
      </header>

      {loading ? (
        <div className="absolute inset-0 flex items-center justify-center text-sm text-acorn-paper/50">
          Gathering your conversations…
        </div>
      ) : stars.length === 0 ? (
        <div className="absolute inset-0 flex items-center justify-center text-sm text-acorn-paper/50">
          No conversations yet — your cloud will grow as you chat.
        </div>
      ) : null}

      {/* Selected detail card. */}
      {selectedStar ? (
        <div className="absolute bottom-6 left-1/2 w-[360px] -translate-x-1/2 rounded-2xl border border-white/10 bg-black/40 p-4 backdrop-blur-xl">
          <div className="mb-1 flex items-start justify-between gap-2">
            <span className="text-sm font-medium text-acorn-paper">{selectedStar.node.title}</span>
            <button
              type="button"
              onClick={() => toggleFavorite(selectedStar.node)}
              title={selectedStar.node.favorite ? "Unstar" : "Star"}
              className="shrink-0 text-acorn-paper/60 hover:text-acorn-orange"
            >
              <Star
                className={cn(
                  "h-4 w-4",
                  selectedStar.node.favorite && "fill-acorn-orange text-acorn-orange",
                )}
              />
            </button>
          </div>
          <div className="mb-3 flex items-center gap-3 text-[11px] text-acorn-paper/50">
            <span className="flex items-center gap-1">
              <MessageSquare className="h-3 w-3" />
              {selectedStar.node.messageCount}
            </span>
            <span>{formatDistanceToNowStrict(new Date(selectedStar.node.lastMessageAt))} ago</span>
          </div>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => onOpen(selectedStar.node.id)}
              className="flex-1 rounded-full bg-acorn-orange px-3 py-1.5 text-xs font-medium text-acorn-paper hover:bg-acorn-brown"
            >
              Open
            </button>
            <button
              type="button"
              onClick={() => toggleArchived(selectedStar.node)}
              className="flex items-center gap-1 rounded-full bg-white/5 px-3 py-1.5 text-xs text-acorn-paper/70 hover:bg-white/10"
            >
              <Archive className="h-3 w-3" />
              {selectedStar.node.archived ? "Unarchive" : "Archive"}
            </button>
          </div>
        </div>
      ) : null}
    </div>
  );
}
