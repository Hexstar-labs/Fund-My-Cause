/**
 * Soroban contract-related types
 * Includes campaign metadata, statistics, and initialization parameters.
 */

/** Campaign status enum variant */
export type CampaignStatus = "Active" | "Successful" | "Refunded" | "Cancelled" | "Paused";

/** Platform configuration */
export interface PlatformConfig {
  address: string;
  feeBps: number;
}

/** Campaign status variant (for UI rendering) */
export type StatusVariant = "active" | "success" | "failed" | "cancelled" | "paused";

/** Single contribution record in transaction history */
export interface ContributionRecord {
  contractId: string;
  contributor: string;
  amount: bigint;
  timestamp: number;
  transactionHash?: string;
}

/** Campaign initialization parameters */
export interface InitializeParams {
  contractId: string;
  creator: string;
  token: string;
  goal: bigint;
  deadline: bigint;
  minContribution: bigint;
  title: string;
  description: string;
  socialLinks?: string[];
  acceptedTokens?: string[];
  platformFeeAddress?: string;
  platformFeeBps?: number;
}

/** High-level campaign metadata from contract */
export interface CampaignInfo {
  contractId: string;
  creator: string;
  token: string;
  goal: bigint;
  deadline: bigint;
  minContribution: bigint;
  /** Maximum contribution per contributor in stroops (0n = no limit) */
  maxContribution: bigint;
  title: string;
  description: string;
  status: CampaignStatus;
  hasPlatformConfig: boolean;
  platformFeeBps: number;
  platformAddress: string;
  socialLinks: string[];
  acceptedTokens?: string[];
}

/** Campaign statistics */
export interface CampaignStats {
  totalRaised: bigint;
  goal: bigint;
  progressBps: number;
  contributorCount: number;
  averageContribution: bigint;
  largestContribution: bigint;
}

/** Normalized campaign data for display */
export interface CampaignData {
  contractId: string;
  title: string;
  description: string;
  raised: number;
  goal: number;
  deadline: string;
  creator: string;
  socialLinks: string[];
  contributorCount: number;
  averageContribution: number;
  status: CampaignStatus;
}
