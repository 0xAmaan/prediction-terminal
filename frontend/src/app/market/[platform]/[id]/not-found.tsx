import Link from "next/link";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Search, ArrowLeft, Home } from "lucide-react";

const MarketNotFound = () => {
  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-4">
      <Card className="max-w-md w-full border-border/30">
        <CardContent className="p-8 text-center">
          {/* Not Found Icon */}
          <div className="h-14 w-14 rounded-full bg-muted flex items-center justify-center mx-auto mb-6">
            <Search className="h-7 w-7 text-muted-foreground" />
          </div>

          {/* Message */}
          <h2 className="text-xl font-semibold mb-2">Market Not Found</h2>
          <p className="text-muted-foreground mb-6">
            The market you&apos;re looking for doesn&apos;t exist or may have been removed.
          </p>

          {/* Actions */}
          <div className="flex flex-col sm:flex-row gap-3 justify-center">
            <Button
              asChild
              variant="default"
              className="flex items-center gap-2"
            >
              <Link href="/">
                <Home className="h-4 w-4" />
                Browse Markets
              </Link>
            </Button>
            <Button
              asChild
              variant="outline"
              className="flex items-center gap-2"
            >
              <Link href="/">
                <ArrowLeft className="h-4 w-4" />
                Go Back
              </Link>
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default MarketNotFound;
