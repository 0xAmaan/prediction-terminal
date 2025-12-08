// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

export const formatPercent = (price: number | string): string => {
  const num = typeof price === "string" ? parseFloat(price) : price;
  return `${Math.round(num * 100)}%`;
};

export const formatVolume = (vol: number | string): string => {
  const num = typeof vol === "string" ? parseFloat(vol) : vol;
  if (num >= 1_000_000) return `$${(num / 1_000_000).toFixed(1)}m Vol.`;
  if (num >= 1_000) return `$${Math.round(num / 1_000)}k Vol.`;
  return `$${Math.round(num)} Vol.`;
};

export const formatCloseTime = (dateStr: string | null): string => {
  if (!dateStr) return "";
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = date.getTime() - now.getTime();
  const diffDays = Math.ceil(diffMs / (1000 * 60 * 60 * 24));
  if (diffDays < 0) return "Ended";
  if (diffDays === 0) return "Today";
  if (diffDays === 1) return "1d";
  if (diffDays < 7) return `${diffDays}d`;
  if (diffDays < 30) return `${Math.floor(diffDays / 7)}w`;
  return `${Math.floor(diffDays / 30)}mo`;
};

export const formatGameTime = (dateStr: string | null): string => {
  if (!dateStr) return "";
  const date = new Date(dateStr);
  return date.toLocaleTimeString("en-US", {
    hour: "numeric",
    minute: "2-digit",
  });
};
