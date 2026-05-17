import { cn } from "@/lib/utils";

interface AcornLogoProps {
  size?: number;
  className?: string;
}

export function AcornLogo({ size = 32, className }: AcornLogoProps) {
  const stemSize = Math.round(size * 0.22);
  const stemOffset = Math.round(size * 0.09);

  return (
    <div
      className={cn("relative bg-acorn-brown", className)}
      style={{
        width: size,
        height: size,
        borderRadius: "50% 50% 45% 45% / 60% 60% 40% 40%",
      }}
      aria-hidden="true"
    >
      <div
        className="absolute left-1/2 -translate-x-1/2 bg-acorn-brown-deep"
        style={{
          top: -stemOffset,
          width: stemSize,
          height: stemSize,
          borderRadius: "40% 40% 50% 50%",
        }}
      />
    </div>
  );
}
