import { Metadata } from "next";

export const metadata: Metadata = {
  title: "Portfolio",
};

export default function PortfolioLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}
