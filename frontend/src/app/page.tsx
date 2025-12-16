import { Hero, Features, Footer } from "@/components/landing";

export default function LandingPage() {
  return (
    <div className="bg-[#070709] min-h-screen text-[#EEF0F1]">
      <Hero />
      <Features />
      <Footer />
    </div>
  );
}
