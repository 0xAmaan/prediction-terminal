"use client";

import Link from "next/link";
import { QRCodeSVG } from "qrcode.react";
import { ArrowLeft } from "lucide-react";
import { PremonitionLogo } from "@/components/icons/premonition-logo";

export default function SignupPage() {
  return (
    <div className="min-h-screen bg-[#070709] flex flex-col">
      {/* Navigation */}
      <nav className="flex items-center justify-between px-6 lg:px-12 py-6">
        <Link href="/" className="flex items-center gap-2">
          <PremonitionLogo size={32} />
          <span className="text-[#EEF0F1] font-semibold text-lg tracking-tight">
            Premonition
          </span>
        </Link>
        <Link
          href="/"
          className="flex items-center gap-2 px-4 py-2 rounded-lg text-[#7D8B96] text-sm font-medium hover:text-[#EEF0F1] transition-colors"
        >
          <ArrowLeft className="w-4 h-4" />
          Back
        </Link>
      </nav>

      {/* Main content */}
      <main className="flex-1 flex flex-col items-center justify-center px-6 pb-24">
        <div className="max-w-md mx-auto text-center space-y-8">
          <h1 className="text-3xl sm:text-4xl font-semibold text-[#EEF0F1] tracking-[-0.02em]">
            Join Premonition
          </h1>
          <p className="text-[#7D8B96] text-lg">
            Scan the QR code below to sign up for early access.
          </p>

          {/* QR Code */}
          <div className="flex justify-center">
            <div className="p-8 rounded-2xl bg-white">
              <QRCodeSVG
                value="https://premonition-waiting-list.vercel.app/"
                size={200}
                level="H"
                includeMargin={false}
              />
            </div>
          </div>

          <p className="text-[#64727C] text-sm">
            Point your camera at the code to continue
          </p>
        </div>
      </main>
    </div>
  );
}
