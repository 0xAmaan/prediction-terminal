import { Metadata } from "next";

type Props = {
  params: Promise<{
    platform: string;
    id: string;
  }>;
};

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { platform, id } = await params;

  try {
    // Fetch market data for metadata
    const apiUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";
    const response = await fetch(`${apiUrl}/api/markets/${platform}/${id}`, {
      // Disable caching for metadata to ensure freshness
      cache: "no-store",
    });

    if (!response.ok) {
      return {
        title: "Market | Premonition",
      };
    }

    const data = await response.json();
    const marketTitle = data.market?.title || "Market";

    return {
      title: `${marketTitle} | Premonition`,
      description: data.market?.description || `Prediction market: ${marketTitle}`,
    };
  } catch (error) {
    console.error("Failed to fetch market metadata:", error);
    return {
      title: "Market | Premonition",
    };
  }
}

export default function MarketLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}
