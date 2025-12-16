"use client";

import { motion } from "framer-motion";
import { Terminal, Database, Cpu } from "lucide-react";

const steps = [
  {
    icon: Terminal,
    step: "01",
    title: "Connect to Markets",
    description: "Real-time WebSocket connections to Polymarket and other prediction market exchanges.",
    command: "$ connect --platform polymarket",
    color: "#54BBF7"
  },
  {
    icon: Database,
    step: "02",
    title: "Analyze Intelligence",
    description: "Process live order books, news feeds, and sentiment data through unified analytics pipeline.",
    command: "$ analyze --depth full --timeframe 24h",
    color: "#4DBE95"
  },
  {
    icon: Cpu,
    step: "03",
    title: "Execute Strategy",
    description: "Place orders, manage positions, and track performance with institutional-grade execution.",
    command: "$ trade --type limit --size 100",
    color: "#F59E0B"
  },
];

export const HowItWorks = () => {
  return (
    <section className="relative py-32 px-4 sm:px-6 lg:px-8 bg-[#070709] overflow-hidden">
      {/* Background effects */}
      <div className="absolute inset-0">
        {/* Gradient orb */}
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[400px] bg-[#54BBF7] opacity-[0.03] blur-[100px] rounded-full" />

        {/* Grid pattern */}
        <div className="absolute inset-0 opacity-[0.02]">
          <div className="absolute inset-0" style={{
            backgroundImage: `
              linear-gradient(rgba(238, 240, 241, 0.05) 1px, transparent 1px),
              linear-gradient(90deg, rgba(238, 240, 241, 0.05) 1px, transparent 1px)
            `,
            backgroundSize: '64px 64px'
          }} />
        </div>
      </div>

      <div className="max-w-7xl mx-auto relative z-10">
        {/* Section header */}
        <div className="text-center mb-24">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6 }}
            viewport={{ once: true }}
            className="space-y-4"
          >
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-[#131419] border border-[rgba(255,255,255,0.06)] text-xs font-mono tracking-wider text-[#7D8B96] mb-6">
              WORKFLOW
            </div>
            <h2 className="text-5xl sm:text-6xl font-semibold text-[#EEF0F1] tracking-tight">
              Simple, powerful workflow
            </h2>
            <p className="text-xl text-[#7D8B96] max-w-2xl mx-auto font-light">
              Three steps from data to execution
            </p>
          </motion.div>
        </div>

        {/* Steps */}
        <div className="relative">
          {/* Connection line */}
          <div className="hidden lg:block absolute top-32 left-[10%] right-[10%] h-[2px]">
            <div className="relative w-full h-full">
              <motion.div
                initial={{ scaleX: 0 }}
                whileInView={{ scaleX: 1 }}
                transition={{ duration: 1.5, delay: 0.5, ease: "easeInOut" }}
                viewport={{ once: true }}
                className="absolute inset-0 origin-left"
                style={{
                  background: 'linear-gradient(90deg, transparent, rgba(84, 187, 247, 0.2), rgba(77, 190, 149, 0.2), rgba(245, 158, 11, 0.2), transparent)'
                }}
              />
            </div>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8 lg:gap-12">
            {steps.map((step, index) => (
              <StepCard key={step.title} step={step} index={index} />
            ))}
          </div>
        </div>

        {/* Bottom info */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.8 }}
          viewport={{ once: true }}
          className="mt-24 text-center"
        >
          <div className="inline-flex items-center gap-3 px-6 py-4 rounded-xl bg-[#131419] border border-[rgba(255,255,255,0.06)]">
            <div className="w-2 h-2 rounded-full bg-[#4DBE95] animate-pulse" />
            <span className="text-[#7D8B96] text-sm">
              <span className="text-[#EEF0F1] font-medium">Live now</span> • All systems operational
            </span>
          </div>
        </motion.div>
      </div>
    </section>
  );
};

// Step card component
const StepCard = ({
  step,
  index,
}: {
  step: typeof steps[0];
  index: number;
}) => {
  return (
    <motion.div
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.6, delay: index * 0.2 }}
      viewport={{ once: true }}
      className="relative group"
    >
      {/* Glow effect */}
      <div
        className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500 blur-2xl -z-10"
        style={{
          background: `radial-gradient(circle at 50% 0%, ${step.color}20, transparent 70%)`
        }}
      />

      <div className="relative">
        {/* Icon circle */}
        <div className="relative mb-8 flex justify-center">
          <div className="absolute inset-0 flex items-center justify-center">
            <div
              className="w-32 h-32 rounded-full opacity-20 blur-2xl"
              style={{ backgroundColor: step.color }}
            />
          </div>
          <div
            className="relative w-24 h-24 rounded-full flex items-center justify-center border-2 transition-all duration-300 group-hover:scale-110 bg-[#070709] z-10"
            style={{ borderColor: `${step.color}40` }}
          >
            <step.icon
              className="w-10 h-10"
              style={{ color: step.color }}
              strokeWidth={1.5}
            />
          </div>

          {/* Step number badge */}
          <div
            className="absolute -top-2 -right-2 w-10 h-10 rounded-full flex items-center justify-center text-xs font-bold font-mono border-2 border-[#070709] bg-[#131419] z-20"
            style={{ color: step.color }}
          >
            {step.step}
          </div>
        </div>

        {/* Content */}
        <div className="text-center space-y-4 px-4">
          <h3 className="text-2xl font-semibold text-[#EEF0F1] tracking-tight">
            {step.title}
          </h3>
          <p className="text-[#7D8B96] text-sm leading-relaxed">
            {step.description}
          </p>

          {/* Terminal command */}
          <div className="mt-6 p-4 rounded-lg bg-[#070709] border border-[rgba(255,255,255,0.05)] font-mono text-left">
            <div className="flex items-center gap-2 mb-2">
              <div className="flex gap-1.5">
                <div className="w-3 h-3 rounded-full bg-[#D84F68]" />
                <div className="w-3 h-3 rounded-full bg-[#F59E0B]" />
                <div className="w-3 h-3 rounded-full bg-[#4DBE95]" />
              </div>
            </div>
            <code className="text-sm" style={{ color: step.color }}>
              {step.command}
            </code>
            <div className="mt-2 flex items-center gap-2">
              <motion.div
                animate={{ opacity: [0, 1, 0] }}
                transition={{ duration: 1.5, repeat: Infinity, ease: "easeInOut" }}
                className="w-2 h-3 bg-[#EEF0F1]"
              />
            </div>
          </div>
        </div>
      </div>
    </motion.div>
  );
};
