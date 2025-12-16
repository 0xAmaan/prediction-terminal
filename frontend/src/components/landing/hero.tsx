"use client";

import Link from "next/link";
import { motion } from "framer-motion";
import { ArrowRight, Activity } from "lucide-react";
import { AnimatedGradient } from "./animated-gradient";

export const Hero = () => {
  return (
    <section className="relative min-h-screen flex flex-col items-center justify-center overflow-hidden px-4 sm:px-6 lg:px-8 pt-20">
      <AnimatedGradient />
      
      <div className="w-full max-w-5xl mx-auto text-center z-10 space-y-8">

        <motion.h1
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
          className="text-5xl sm:text-6xl md:text-7xl lg:text-8xl font-semibold tracking-tight text-[#EEF0F1]"
        >
          Real-Time Prediction
          <br />
          <span className="text-transparent bg-clip-text bg-gradient-to-r from-[#54BBF7] to-[#4DBE95]">
            Market Intelligence
          </span>
        </motion.h1>

        <motion.p
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.2 }}
          className="text-lg sm:text-xl text-[#7D8B96] max-w-2xl mx-auto leading-relaxed"
        >
          Aggregate and analyze prediction markets from Polymarket in one powerful terminal.
          Track whales, analyze sentiment, and execute trades with precision.
        </motion.p>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.3 }}
          className="flex flex-col sm:flex-row items-center justify-center gap-4"
        >
          <Link
            href="/markets"
            className="group relative px-8 py-4 rounded-full bg-[#EEF0F1] text-[#070709] font-medium text-lg transition-transform hover:scale-105 active:scale-95"
          >
            Explore Markets
            <ArrowRight className="inline-block ml-2 w-5 h-5 transition-transform group-hover:translate-x-1" />
          </Link>
        </motion.div>

        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 1, delay: 0.8 }}
          className="pt-12 flex items-center justify-center gap-8 text-[#7D8B96] text-sm"
        >
        </motion.div>
      </div>
    </section>
  );
};
