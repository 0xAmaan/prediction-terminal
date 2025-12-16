"use client";

import { motion } from "framer-motion";

const features = [
  {
    title: "Live Data",
    description:
      "WebSocket streams from Polymarket. Order books, trades, and prices update in real-time with sub-second latency.",
  },
  {
    title: "AI Research",
    description:
      "GPT-4 powered analysis that breaks down markets, searches the web, and generates trading-focused research reports.",
  },
  {
    title: "Direct Trading",
    description:
      "Execute trades directly on Polymarket. EIP-712 signed orders with your own wallet. Non-custodial.",
  },
];

export const Features = () => {
  return (
    <section className="py-24 px-6 lg:px-12 bg-[#070709] border-t border-[#1e1f25]/50">
      <div className="max-w-6xl mx-auto">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-12 md:gap-8">
          {features.map((feature, index) => (
            <motion.div
              key={feature.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
              viewport={{ once: true }}
              className="space-y-3"
            >
              <h3 className="text-lg font-semibold text-[#EEF0F1]">
                {feature.title}
              </h3>
              <p className="text-sm text-[#7D8B96] leading-relaxed">
                {feature.description}
              </p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
};
