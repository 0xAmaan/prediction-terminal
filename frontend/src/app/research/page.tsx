import { redirect } from "next/navigation";

// Redirect to the markets page with the research tab active
export default function ResearchPage() {
  redirect("/markets?tab=research");
}
