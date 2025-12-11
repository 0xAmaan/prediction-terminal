"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Loader2, Sparkles } from "lucide-react";
import { useResearch } from "@/hooks/use-research";
import { ResearchModal } from "./research-modal";

interface ResearchButtonProps {
  platform: string;
  marketId: string;
  marketTitle: string;
}

export function ResearchButton({
  platform,
  marketId,
  marketTitle,
}: ResearchButtonProps) {
  const [isOpen, setIsOpen] = useState(false);
  const { job, isLoading, error, startResearch, reset } = useResearch();

  const handleClick = async () => {
    setIsOpen(true);
    if (!job) {
      try {
        await startResearch(platform, marketId);
      } catch (e) {
        console.error("Failed to start research:", e);
      }
    }
  };

  const handleClose = () => {
    setIsOpen(false);
    // Reset job state when modal closes so new research can be started
    if (job?.status === "completed" || job?.status === "failed") {
      reset();
    }
  };

  return (
    <>
      <Button
        onClick={handleClick}
        disabled={isLoading}
        variant="outline"
        className="gap-2"
      >
        {isLoading ? (
          <Loader2 className="h-4 w-4 animate-spin" />
        ) : (
          <Sparkles className="h-4 w-4" />
        )}
        Research
      </Button>

      <ResearchModal
        isOpen={isOpen}
        onClose={handleClose}
        job={job}
        marketTitle={marketTitle}
        error={error}
      />
    </>
  );
}
