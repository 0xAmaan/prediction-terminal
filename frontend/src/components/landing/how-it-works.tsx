"use client";

import { motion } from "framer-motion";
import { Search, BarChart2, MousePointerClick } from "lucide-react";

const steps = [
  {
    icon: Search,
    title: "Browse Markets",
    description: "Explore active prediction markets from Polymarket across politics, crypto, sports, and more.",
  },
  {
    icon: BarChart2,
    title: "Analyze Data",
    description: "Dive deep with real-time charts, order book analysis, and AI-generated research reports.",
  },
  {
    icon: MousePointerClick,
    title: "Execute Trades",
    description: "Connect your wallet and trade directly on the platform with instant execution.",
  },
];

export const HowItWorks = () => {
  return (
    <section className="py-24 px-4 sm:px-6 lg:px-8 bg-[#070709] relative overflow-hidden">
      {/* Background decoration */}
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[1000px] h-[400px] bg-[#54BBF7] opacity-[0.03] blur-[120px] rounded-full" />

      <div className="max-w-7xl mx-auto relative z-10">
        <div className="text-center mb-16">
          <h2 className="text-3xl sm:text-4xl font-semibold text-[#EEF0F1] mb-4">
            How it works
          </h2>
        </div>

        <div className="relative grid grid-cols-1 md:grid-cols-3 gap-12">
          {/* Connecting line for desktop */}
          <div className="hidden md:block absolute top-12 left-[16%] right-[16%] h-[2px] bg-gradient-to-r from-transparent via-[#202427] to-transparent" />

          {steps.map((step, index) => (
            <motion.div
              key={step.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: index * 0.2 }}
              viewport={{ once: true }}
              className="relative flex flex-col items-center text-center group"
            >
              <div className="w-24 h-24 rounded-full bg-[#131419] border border-[rgba(255,255,255,0.06)] flex items-center justify-center mb-6 z-10 transition-transform duration-300 group-hover:scale-110 group-hover:border-[#54BBF7]/30 shadow-lg shadow-black/50">
                <step.icon className="w-10 h-10 text-[#54BBF7]" />
              </div>
              
              {/* Step number badge removed */}


              <h3 className="text-xl font-medium text-[#EEF0F1] mb-3">
                {step.title}
              </h3>
              <p className="text-[#7D8B96] text-sm leading-relaxed max-w-xs">
                {step.description}
              </p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
};
