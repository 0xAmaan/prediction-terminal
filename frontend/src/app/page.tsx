import { Metadata } from "next";
import { Hero, Features, Footer } from "@/components/landing";

export const metadata: Metadata = {
  title: "Premonition - Prediction Market Terminal",
};

export default function LandingPage() {
  return (
    <div className="bg-[#070709] min-h-screen text-[#EEF0F1]">
      <Hero />
      <Features />
      <Footer />
    </div>
  );
}
