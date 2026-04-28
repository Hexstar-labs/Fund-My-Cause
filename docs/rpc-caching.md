# RPC Caching Strategy

Soroban RPC calls are expensive (network round-trip + simulation). The caching layer reduces redundant calls by storing results in memory with two tiers.

## Architecture

```
Component
  └── React Query (staleTime / gcTime)
        └── useCampaign / getCampaignInfo / getCampaignStats
              └── simulateView  ← checks rpc-cache before hitting RPC
                    └── Soroban RPC (only on cache miss)
```

## Two Cache Tiers

### TTL Cache — live data

| Setting | Value |
|---|---|
| Default TTL | 30 seconds |
| React Query `staleTime` | 30 s |
| React Query `gcTime` | 5 minutes |
| Invalidated by | `cacheInvalidateLive(contractId)` after any mutating tx |

Methods cached with TTL: `get_stats`, `total_raised`, `contribution`, and any other method not in the static list.

### Static Cache — immutable data

| Setting | Value |
|---|---|
| TTL | Indefinite (`Infinity`) |
| React Query `staleTime` | `Infinity` (query key: `["campaign-info"]`) |
| React Query `gcTime` | 1 hour |
| Invalidated by | `cacheInvalidate(contractId)` (full wipe, rarely needed) |

Methods cached statically: `goal`, `deadline`, `min_contribution`, `max_contribution`, `title`, `description`, `creator`, `version`, `social_links`.

These values are set at contract initialization and never change.

## Cache Key Format

```
${contractId}:${method}
```

Example: `CAABC...XYZ:title`

Calls with arguments (e.g. `contribution(address)`) bypass the cache because the key would need to encode args. Extend `cacheGet`/`cacheSet` with an `argsKey` if per-contributor caching is needed.

## Invalidation

After any mutating transaction (`contribute`, `withdraw`, `refund_single`, etc.) confirms on-chain, `invokeContract` calls `cacheInvalidateLive(contractId)` to drop all TTL entries for that contract. Static entries are preserved.

## Cache Statistics

```ts
import { cacheStats } from "@/lib/rpc-cache";

const { hits, misses, size, staticSize } = cacheStats();
```

Useful for debugging or exposing a dev-mode cache inspector.

## Files

| File | Role |
|---|---|
| `src/lib/rpc-cache.ts` | Cache store, get/set/invalidate/stats |
| `src/lib/contract.ts` | `simulateView` reads/writes cache; `invokeContract` invalidates on success |
| `src/context/ReactQueryProvider.tsx` | React Query stale/gc times aligned with cache TTLs |
