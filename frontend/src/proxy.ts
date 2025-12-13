import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

// KALSHI_DISABLED: Redirect Kalshi market URLs to home while focusing on Polymarket
export default function proxy(request: NextRequest) {
  if (request.nextUrl.pathname.startsWith("/market/kalshi")) {
    return NextResponse.redirect(new URL("/", request.url));
  }
  return NextResponse.next();
}

export const config = {
  matcher: "/market/kalshi/:path*",
};
