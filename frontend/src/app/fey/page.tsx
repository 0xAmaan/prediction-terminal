"use client";

import { Heart, User, CreditCard, MessageCircle, Keyboard } from "lucide-react";

/**
 * Fey UI Kit Design Tokens (from FEY-UI-KIT-REFERENCE.md)
 *
 * Colors:
 * - BG-100: #070709 (deepest)
 * - BG-200: #101116
 * - BG-300: #131419 (card background)
 * - Grey-100: #EEF0F1 (text)
 * - Grey-500: #7D8B96 (muted text)
 * - Grey-900: #202427 (darkest grey)
 *
 * Accent Colors:
 * - Alert red: #D84F68
 * - Earth brown: #C27C58
 * - Sky blue: #54BBF7
 * - Teal: #4DBE95
 * - Royal blue: #6166DC
 *
 * Typography:
 * - Font: Calibre (using Inter as fallback)
 * - Heading: Semibold, 18px, -2% letter spacing
 * - Body: Regular, 16px
 */

// Fey color tokens
const fey = {
  bg: {
    100: "#070709",
    200: "#101116",
    300: "#131419",
    400: "#16181C",
    500: "#1A1B20",
  },
  grey: {
    100: "#EEF0F1",
    500: "#7D8B96",
    900: "#202427",
  },
  accent: {
    red: "#D84F68",
    brown: "#C27C58",
    blue: "#54BBF7",
    teal: "#4DBE95",
    purple: "#6166DC",
  },
};

// Icon background colors with 10% opacity
const iconBgColors = {
  feedback: "rgba(216, 79, 104, 0.1)",
  profile: "rgba(194, 124, 88, 0.1)",
  payment: "rgba(84, 187, 247, 0.1)",
  communication: "rgba(77, 190, 149, 0.1)",
  shortcuts: "rgba(97, 102, 220, 0.1)",
};

type PreferenceCardProps = {
  icon: React.ReactNode;
  iconBg: string;
  title: string;
  description: string;
  isHovered?: boolean;
};

const PreferenceCard = ({
  icon,
  iconBg,
  title,
  description,
}: PreferenceCardProps) => {
  return (
    <div
      className="group relative flex items-center justify-between overflow-hidden rounded-lg px-6 py-5 transition-all duration-200 hover:bg-opacity-80"
      style={{ backgroundColor: fey.bg[300] }}
    >
      <div className="flex items-center gap-6">
        {/* Icon */}
        <div
          className="flex items-center justify-center rounded-lg p-1.5"
          style={{ backgroundColor: iconBg }}
        >
          {icon}
        </div>

        {/* Content */}
        <div className="flex flex-col gap-3">
          <h3
            className="text-lg font-semibold tracking-tight"
            style={{ color: fey.grey[100], letterSpacing: "-0.36px" }}
          >
            {title}
          </h3>
          <p className="text-base" style={{ color: fey.grey[500] }}>
            {description}
          </p>
        </div>
      </div>

      {/* Hover indicator */}
      <div className="h-full w-1 rounded-full bg-white/10 opacity-0 transition-opacity group-hover:opacity-100" />
    </div>
  );
};

// Command/Shortcut tag component
const CommandTag = ({ children }: { children: string }) => (
  <span
    className="inline-flex items-center justify-center rounded px-1.5 py-1 text-xs font-medium"
    style={{
      backgroundColor: fey.grey[900],
      color: fey.grey[100],
      border: "0.5px solid rgba(255, 255, 255, 0.1)",
    }}
  >
    {children}
  </span>
);

// Preference card with keyboard shortcut
const ShortcutPreferenceCard = () => {
  return (
    <div
      className="group relative flex items-center justify-between overflow-hidden rounded-lg px-6 py-5 transition-all duration-200 hover:bg-opacity-80"
      style={{ backgroundColor: fey.bg[300] }}
    >
      <div className="flex items-center gap-6">
        {/* Icon */}
        <div
          className="flex items-center justify-center rounded-lg p-1.5"
          style={{ backgroundColor: iconBgColors.shortcuts }}
        >
          <Keyboard className="h-[18px] w-[18px]" style={{ color: fey.accent.purple }} />
        </div>

        {/* Content */}
        <div className="flex flex-col gap-3">
          <h3
            className="text-lg font-semibold tracking-tight"
            style={{ color: fey.grey[100], letterSpacing: "-0.36px" }}
          >
            Shortcuts
          </h3>
          <p className="flex items-center gap-1.5 text-base" style={{ color: fey.grey[500] }}>
            Press <CommandTag>?</CommandTag> anytime for a cheat sheet.
          </p>
        </div>
      </div>
    </div>
  );
};

// Tag component
type TagProps = {
  children: string;
  variant?: "default" | "orange" | "green" | "blue";
};

const Tag = ({ children, variant = "default" }: TagProps) => {
  const variantStyles = {
    default: { bg: "rgba(255, 255, 255, 0.05)", border: "#c6c6c6", text: fey.grey[100] },
    orange: { bg: "rgba(194, 124, 88, 0.15)", border: fey.accent.brown, text: fey.accent.brown },
    green: { bg: "rgba(77, 190, 149, 0.15)", border: fey.accent.teal, text: fey.accent.teal },
    blue: { bg: "rgba(84, 187, 247, 0.15)", border: fey.accent.blue, text: fey.accent.blue },
  };

  const style = variantStyles[variant];

  return (
    <span
      className="inline-flex items-center rounded-full px-3 py-1 text-xs font-medium"
      style={{
        backgroundColor: style.bg,
        border: `1px solid ${style.border}`,
        color: style.text,
      }}
    >
      {children}
    </span>
  );
};

// Button component
type ButtonProps = {
  children: React.ReactNode;
  variant?: "default" | "ghost";
  onClick?: () => void;
};

const Button = ({ children, variant = "default" }: ButtonProps) => {
  const baseStyles = "inline-flex items-center justify-center gap-2 rounded-full px-4 py-2 text-sm font-normal transition-all duration-200";

  if (variant === "ghost") {
    return (
      <button
        className={`${baseStyles} hover:bg-white/5`}
        style={{ color: "rgba(255, 255, 255, 0.68)" }}
      >
        {children}
      </button>
    );
  }

  return (
    <button
      className={baseStyles}
      style={{
        backgroundColor: "rgba(255, 255, 255, 0.05)",
        border: "1px solid #c6c6c6",
        color: "rgba(255, 255, 255, 0.68)",
        boxShadow: "-0.5px 1.5px 1.5px 0px rgba(0, 0, 0, 0.25)",
      }}
    >
      {children}
    </button>
  );
};

const FeyPage = () => {
  return (
    <div
      className="min-h-screen p-8"
      style={{ backgroundColor: fey.bg[100] }}
    >
      <div className="mx-auto max-w-4xl space-y-12">
        {/* Header */}
        <div className="space-y-2">
          <h1
            className="text-3xl font-semibold"
            style={{ color: fey.grey[100] }}
          >
            Fey UI Kit Mockups
          </h1>
          <p style={{ color: fey.grey[500] }}>
            Testing components from the Fey UI Kit design system
          </p>
        </div>

        {/* Section: Preference Cards */}
        <section className="space-y-4">
          <div className="flex items-center gap-3">
            <h2
              className="text-xs font-medium uppercase tracking-widest"
              style={{ color: fey.accent.blue }}
            >
              Preference Cards
            </h2>
            <div className="h-px flex-1" style={{ backgroundColor: "rgba(255, 255, 255, 0.1)" }} />
          </div>

          <div className="space-y-3">
            <PreferenceCard
              icon={<User className="h-[18px] w-[18px]" style={{ color: fey.accent.brown }} />}
              iconBg={iconBgColors.profile}
              title="vaibhav agrawal"
              description="vaibhavagrawal2907@gmail.com"
            />

            <PreferenceCard
              icon={<CreditCard className="h-[18px] w-[18px]" style={{ color: fey.accent.blue }} />}
              iconBg={iconBgColors.payment}
              title="Payment method"
              description="Visa **** 3111"
            />

            <PreferenceCard
              icon={<MessageCircle className="h-[18px] w-[18px]" style={{ color: fey.accent.teal }} />}
              iconBg={iconBgColors.communication}
              title="Communication"
              description="Manage your email newsletter, get help, or join our Slack."
            />

            <ShortcutPreferenceCard />

            <PreferenceCard
              icon={<Heart className="h-[18px] w-[18px]" style={{ color: fey.accent.red }} />}
              iconBg={iconBgColors.feedback}
              title="Share feedback"
              description="Bugs, suggestions or simple hello?"
            />
          </div>
        </section>

        {/* Section: Tags */}
        <section className="space-y-4">
          <div className="flex items-center gap-3">
            <h2
              className="text-xs font-medium uppercase tracking-widest"
              style={{ color: fey.accent.teal }}
            >
              Tags
            </h2>
            <div className="h-px flex-1" style={{ backgroundColor: "rgba(255, 255, 255, 0.1)" }} />
          </div>

          <div className="flex flex-wrap gap-3">
            <Tag>Default</Tag>
            <Tag variant="orange">Early Adopter</Tag>
            <Tag variant="green">Active</Tag>
            <Tag variant="blue">Premium</Tag>
            <CommandTag>?</CommandTag>
            <CommandTag>⌘</CommandTag>
            <CommandTag>K</CommandTag>
          </div>
        </section>

        {/* Section: Buttons */}
        <section className="space-y-4">
          <div className="flex items-center gap-3">
            <h2
              className="text-xs font-medium uppercase tracking-widest"
              style={{ color: fey.accent.purple }}
            >
              Buttons
            </h2>
            <div className="h-px flex-1" style={{ backgroundColor: "rgba(255, 255, 255, 0.1)" }} />
          </div>

          <div className="flex flex-wrap gap-3">
            <Button>Keyboard shortcuts</Button>
            <Button>Log Out</Button>
            <Button variant="ghost">Update payment method</Button>
          </div>
        </section>

        {/* Section: Card Example */}
        <section className="space-y-4">
          <div className="flex items-center gap-3">
            <h2
              className="text-xs font-medium uppercase tracking-widest"
              style={{ color: fey.accent.red }}
            >
              Card with Content
            </h2>
            <div className="h-px flex-1" style={{ backgroundColor: "rgba(255, 255, 255, 0.1)" }} />
          </div>

          <div
            className="overflow-hidden rounded-xl p-8"
            style={{
              backgroundColor: fey.bg[200],
              border: "1px solid rgba(255, 255, 255, 0.06)",
            }}
          >
            <div className="flex items-start justify-between">
              <div className="space-y-4">
                <div className="flex items-center gap-2">
                  <Tag variant="orange">Early Adopter</Tag>
                </div>
                <div>
                  <h3
                    className="text-2xl font-semibold"
                    style={{ color: fey.grey[100] }}
                  >
                    Visa •••• 1111
                  </h3>
                  <p className="mt-1" style={{ color: fey.grey[500] }}>
                    Expires June 2031
                  </p>
                </div>
                <Button>Update payment method</Button>
                <p className="text-sm" style={{ color: fey.grey[500] }}>
                  Next payment: Sep 8, 2024
                </p>
              </div>
            </div>
          </div>
        </section>

        {/* Color Palette Reference */}
        <section className="space-y-4">
          <div className="flex items-center gap-3">
            <h2
              className="text-xs font-medium uppercase tracking-widest"
              style={{ color: fey.grey[500] }}
            >
              Color Palette
            </h2>
            <div className="h-px flex-1" style={{ backgroundColor: "rgba(255, 255, 255, 0.1)" }} />
          </div>

          <div className="grid grid-cols-5 gap-3">
            {/* Backgrounds */}
            {Object.entries(fey.bg).map(([key, value]) => (
              <div key={key} className="space-y-2">
                <div
                  className="h-16 rounded-lg border"
                  style={{ backgroundColor: value, borderColor: "rgba(255,255,255,0.1)" }}
                />
                <div className="text-xs" style={{ color: fey.grey[500] }}>
                  <div>BG-{key}</div>
                  <div className="font-mono">{value}</div>
                </div>
              </div>
            ))}
          </div>

          <div className="grid grid-cols-5 gap-3 pt-4">
            {/* Accents */}
            {Object.entries(fey.accent).map(([key, value]) => (
              <div key={key} className="space-y-2">
                <div
                  className="h-16 rounded-lg"
                  style={{ backgroundColor: value }}
                />
                <div className="text-xs" style={{ color: fey.grey[500] }}>
                  <div className="capitalize">{key}</div>
                  <div className="font-mono">{value}</div>
                </div>
              </div>
            ))}
          </div>
        </section>
      </div>
    </div>
  );
};

export default FeyPage;
