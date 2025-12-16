"use client";

import { motion } from "framer-motion";
import { Zap, Brain, Globe, TrendingUp } from "lucide-react";

const features = [
  {
    icon: Zap,
    title: "Real-Time Data",
    description: "Live order books and trade feeds via direct WebSocket connections. Millisecond latency for precise execution.",
    color: "#54BBF7", // Sky Blue
  },
  {
    icon: Brain,
    title: "Deep Research",
    description: "AI-powered market analysis using GPT-4 to synthesize news, sentiment, and historical data into actionable reports.",
    color: "#6166DC", // Purple
  },
  {
    icon: Globe,
    title: "News Intelligence",
    description: "Aggregated news from 25+ global sources with relevance filtering and sentiment scoring.",
    color: "#4DBE95", // Teal
  },
  {
    icon: TrendingUp,
    title: "Trading Ready",
    description: "Direct execution on Polymarket with advanced order types and position management tools.",
    color: "#D84F68", // Red
  },
];

export const Features = () => {
  return (
    <section className="py-24 px-4 sm:px-6 lg:px-8 relative">
      <div className="max-w-7xl mx-auto">
        <div className="text-center mb-16">
          <h2 className="text-3xl sm:text-4xl font-semibold text-[#EEF0F1] mb-4">
            Everything you need to trade
          </h2>
          <p className="text-[#7D8B96] max-w-2xl mx-auto">
            A complete suite of tools designed for the modern prediction market trader.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {features.map((feature, index) => (
            <motion.div
              key={feature.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
              viewport={{ once: true }}
              className="group p-6 rounded-2xl bg-[#131419] border border-[rgba(255,255,255,0.06)] hover:bg-[#16181C] hover:border-[rgba(255,255,255,0.1)] transition-all duration-300"
            >
              <div 
                className="w-12 h-12 rounded-xl flex items-center justify-center mb-4 transition-colors group-hover:bg-opacity-20"
                style={{ backgroundColor: `${feature.color}1A` }}
              >
                <feature.icon 
                  className="w-6 h-6 transition-transform group-hover:scale-110" 
                  style={{ color: feature.color }} 
                />
              </div>
              <h3 className="text-xl font-medium text-[#EEF0F1] mb-2">
                {feature.title}
              </h3>
              <p className="text-[#7D8B96] text-sm leading-relaxed">
                {feature.description}
              </p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
};
