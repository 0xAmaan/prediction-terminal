"use client";

import { useEffect } from "react";
import Link from "next/link";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { AlertCircle, ArrowLeft, RotateCcw } from "lucide-react";

interface ErrorProps {
  error: Error & { digest?: string };
  reset: () => void;
}

const MarketError = ({ error, reset }: ErrorProps) => {
  useEffect(() => {
    // Log error to console in development
    console.error("Market page error:", error);
  }, [error]);

  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-4">
      <Card className="max-w-md w-full border-border/30">
        <CardContent className="p-8 text-center">
          {/* Error Icon */}
          <div className="h-14 w-14 rounded-full bg-destructive/20 flex items-center justify-center mx-auto mb-6">
            <AlertCircle className="h-7 w-7 text-destructive" />
          </div>

          {/* Error Message */}
          <h2 className="text-xl font-semibold mb-2">Something went wrong</h2>
          <p className="text-muted-foreground mb-6">
            {error.message || "Failed to load market data. Please try again."}
          </p>

          {/* Actions */}
          <div className="flex flex-col sm:flex-row gap-3 justify-center">
            <Button
              onClick={reset}
              variant="default"
              className="flex items-center gap-2"
            >
              <RotateCcw className="h-4 w-4" />
              Try Again
            </Button>
            <Button
              asChild
              variant="outline"
              className="flex items-center gap-2"
            >
              <Link href="/">
                <ArrowLeft className="h-4 w-4" />
                Back to Markets
              </Link>
            </Button>
          </div>

          {/* Error Details (dev only) */}
          {process.env.NODE_ENV === "development" && error.digest && (
            <p className="text-xs text-muted-foreground mt-6 font-mono">
              Error ID: {error.digest}
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  );
};

export default MarketError;
