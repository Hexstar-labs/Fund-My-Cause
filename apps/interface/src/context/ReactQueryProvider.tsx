"use client";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";

// Mirrors the two cache tiers in src/lib/rpc-cache.ts:
//  - "campaign" queries (live stats)  → stale after 30 s, gc after 5 min
//  - "campaign-info" queries (static) → never stale, gc after 1 hour
const makeClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 30_000, // live data: 30 s
        gcTime: 5 * 60_000, // 5 min
        retry: false,
      },
    },
  });

export function ReactQueryProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const [client] = useState(() => {
    const qc = makeClient();

    // Static campaign info (goal, deadline, title…) — never re-fetch
    qc.setQueryDefaults(["campaign-info"], {
      staleTime: Infinity,
      gcTime: 60 * 60_000, // 1 hour
    });

    return qc;
  });

  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}
