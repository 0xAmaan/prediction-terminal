"use client";

import { Toaster as Sonner, type ToasterProps } from "sonner";

// Fey design tokens
const fey = {
  bg100: "#070709",
  bg300: "#131419",
  bg400: "#16181C",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  red: "#D84F68",
  yellow: "#F5A623",
  border: "rgba(255, 255, 255, 0.06)",
};

const Toaster = ({ ...props }: ToasterProps) => {
  return (
    <Sonner
      theme="dark"
      position="bottom-right"
      className="toaster group"
      toastOptions={{
        style: {
          background: fey.bg300,
          border: `1px solid ${fey.border}`,
          color: fey.grey100,
        },
        classNames: {
          success: "!border-[#4DBE95]/30",
          error: "!border-[#D84F68]/30",
          loading: "!border-[#F5A623]/30",
        },
      }}
      {...props}
    />
  );
};

export { Toaster }
