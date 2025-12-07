import { Badge } from "@/components/ui/badge";
import type { Platform } from "@/lib/types";

interface PlatformBadgeProps {
  platform: Platform;
  size?: "sm" | "md";
}

export const PlatformBadge = ({ platform, size = "md" }: PlatformBadgeProps) => {
  const sizeClasses = size === "sm"
    ? "text-sm px-1.5 py-0.5"
    : "text-base px-2.5 py-1";

  if (platform === "kalshi") {
    return (
      <Badge
        variant="outline"
        className={`bg-[#22c55e]/15 text-[#22c55e] border-[#22c55e]/40 font-semibold tracking-wide ${sizeClasses}`}
      >
        Kalshi
      </Badge>
    );
  }

  return (
    <Badge
      variant="outline"
      className={`bg-[#3b82f6]/15 text-[#3b82f6] border-[#3b82f6]/40 font-semibold tracking-wide ${sizeClasses}`}
    >
      Poly
    </Badge>
  );
};
