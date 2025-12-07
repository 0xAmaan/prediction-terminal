import { Skeleton } from "@/components/ui/skeleton";

const MarketLoading = () => {
  return (
    <div className="min-h-screen bg-background">
      {/* Header Skeleton */}
      <header className="border-b border-border/50 bg-card/50">
        <div className="max-w-6xl mx-auto px-6 py-4">
          <div className="flex items-center gap-4">
            <Skeleton className="h-9 w-9 rounded-lg" />
            <div className="flex-1">
              <div className="flex items-center gap-3 mb-2">
                <Skeleton className="h-5 w-16" />
                <Skeleton className="h-5 w-14" />
              </div>
              <Skeleton className="h-6 w-96 max-w-full" />
            </div>
            <Skeleton className="h-10 w-36 hidden sm:block" />
          </div>
        </div>
      </header>

      {/* Main Content Skeleton */}
      <main className="max-w-6xl mx-auto px-6 py-8">
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Left Column */}
          <div className="lg:col-span-2 space-y-6">
            {/* Price Cards */}
            <div className="grid grid-cols-2 gap-4">
              <Skeleton className="h-32 rounded-xl" />
              <Skeleton className="h-32 rounded-xl" />
            </div>

            {/* Chart */}
            <Skeleton className="h-80 rounded-xl" />

            {/* Order Book */}
            <Skeleton className="h-64 rounded-xl" />

            {/* Description */}
            <Skeleton className="h-40 rounded-xl" />
          </div>

          {/* Right Column */}
          <div className="space-y-6">
            {/* Market Stats */}
            <Skeleton className="h-48 rounded-xl" />

            {/* Trade History */}
            <Skeleton className="h-48 rounded-xl" />

            {/* User Position */}
            <Skeleton className="h-48 rounded-xl" />
          </div>
        </div>
      </main>
    </div>
  );
};

export default MarketLoading;
