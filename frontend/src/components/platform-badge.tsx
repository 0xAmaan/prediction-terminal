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

  // KALSHI_DISABLED: removed Kalshi conditional branch
  return (
    <Badge
      variant="outline"
      className={`bg-[#3b82f6]/15 text-[#3b82f6] border-[#3b82f6]/40 font-semibold tracking-wide ${sizeClasses}`}
    >
      Polymarket
    </Badge>
  );
};
