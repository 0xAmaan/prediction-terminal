import { Metadata } from "next";

export const metadata: Metadata = {
  title: "Markets",
};

export default function MarketsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}
