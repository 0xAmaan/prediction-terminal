"use client";

import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { Sparkles } from "lucide-react";

interface ResearchButtonProps {
  platform: string;
  marketId: string;
  marketTitle: string;
}

export function ResearchButton({
  platform,
  marketId,
}: ResearchButtonProps) {
  const router = useRouter();

  const handleClick = () => {
    router.push(`/research/${platform}-${marketId}`);
  };

  return (
    <Button
      onClick={handleClick}
      variant="outline"
      className="gap-2"
    >
      <Sparkles className="h-4 w-4" />
      Research
    </Button>
  );
}
