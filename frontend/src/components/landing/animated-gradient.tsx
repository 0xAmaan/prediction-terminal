"use client";

import { motion } from "framer-motion";

export const AnimatedGradient = () => {
  return (
    <div className="absolute inset-0 -z-10 overflow-hidden">
      <div className="absolute inset-0 bg-[#070709]" />
      <motion.div
        className="absolute -top-[40%] -left-[20%] w-[70%] h-[70%] rounded-full opacity-20 blur-[120px]"
        style={{
          background: "radial-gradient(circle, #54BBF7 0%, transparent 70%)",
        }}
        animate={{
          x: [0, 100, 0],
          y: [0, -50, 0],
          scale: [1, 1.2, 1],
        }}
        transition={{
          duration: 20,
          repeat: Infinity,
          ease: "linear",
        }}
      />
      <motion.div
        className="absolute top-[20%] -right-[20%] w-[60%] h-[60%] rounded-full opacity-20 blur-[120px]"
        style={{
          background: "radial-gradient(circle, #4DBE95 0%, transparent 70%)",
        }}
        animate={{
          x: [0, -100, 0],
          y: [0, 50, 0],
          scale: [1, 1.1, 1],
        }}
        transition={{
          duration: 15,
          repeat: Infinity,
          ease: "linear",
        }}
      />
      <motion.div
        className="absolute -bottom-[20%] left-[20%] w-[50%] h-[50%] rounded-full opacity-15 blur-[100px]"
        style={{
          background: "radial-gradient(circle, #6166DC 0%, transparent 70%)",
        }}
        animate={{
          x: [0, 50, 0],
          y: [0, 50, 0],
          scale: [1, 1.3, 1],
        }}
        transition={{
          duration: 18,
          repeat: Infinity,
          ease: "linear",
        }}
      />
    </div>
  );
};
