/**
 * Contract interaction layer for the Fund-My-Cause crowdfunding contract.
 * Wraps all Soroban RPC calls using @stellar/stellar-sdk.
 */

import {
  Contract,
  TransactionBuilder,
  BASE_FEE,
  Account,
  nativeToScVal,
  Address,
  scValToNative,
  rpc as SorobanRpc,
  Horizon,
} from "@stellar/stellar-sdk";
import {
  CONTRACT_ID,
  RPC_URL,
  NETWORK_PASSPHRASE,
  HORIZON_URL,
} from "@/lib/constants";
import { isValidContractId } from "@/lib/validation";
import type { SignFn } from "@/types/contract";
import { ContractError } from "@/types/contract";
import { cacheGet, cacheSet, cacheInvalidateLive } from "@/lib/rpc-cache";

// Re-export types for backward compatibility
export type { SignFn } from "@/types/contract";
export { ContractError } from "@/types/contract";

// ── Internal helpers ──────────────────────────────────────────────────────────

/**
 * Returns a configured Soroban RPC server instance.
 * @param {string} [rpcUrl=RPC_URL] - Optional override; defaults to NEXT_PUBLIC_RPC_URL env var
 * @returns {SorobanRpc.Server} Configured Soroban RPC server
 */
export function getContractClient(rpcUrl: string = RPC_URL): SorobanRpc.Server {
  return new SorobanRpc.Server(rpcUrl);
}

/**
 * Simulates a read-only contract call and returns the decoded native value.
 * Uses a dummy account — no signing required.
 * @param {string} contractId - The Soroban contract address
 * @param {string} method - Contract method name to call
 * @param {any[]} [args=[]] - Method arguments
 * @returns {Promise<unknown>} Decoded return value from the contract
 * @throws {ContractError} If simulation fails
 */
async function simulateView(
  contractId: string,
  method: string,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  args: any[] = [],
): Promise<unknown> {
  if (!isValidContractId(contractId)) {
    throw new ContractError(`Invalid contract ID format: ${contractId}`);
  }

  // Return cached value when available (args-less calls only)
  if (args.length === 0) {
    const cached = cacheGet(contractId, method);
    if (cached !== undefined) return cached;
  }

  const rpc = getContractClient();
  const contract = new Contract(contractId);
  // Dummy account — only used for simulation, never submitted.
  const account = new Account(
    "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN",
    "0",
  );
  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call(method, ...args))
    .setTimeout(30)
    .build();

  const result = await rpc.simulateTransaction(tx);
  if (SorobanRpc.Api.isSimulationError(result)) {
    throw new ContractError(result.error);
  }
  const sim = result as SorobanRpc.Api.SimulateTransactionSuccessResponse;
  const value = scValToNative(sim.result!.retval);

  if (args.length === 0) cacheSet(contractId, method, value);

  return value;
}

/**
 * Builds, prepares, signs, submits, and polls a state-changing contract call.
 * Returns the transaction hash on success.
 * @param {string} caller - The caller's Stellar public key
 * @param {string} contractId - The Soroban contract address
 * @param {string} method - Contract method name to call
 * @param {any[]} args - Method arguments
 * @param {SignFn} signTx - Wallet signing function
 * @returns {Promise<string>} Transaction hash on success
 * @throws {ContractError} If submission or confirmation fails
 */
async function invokeContract(
  caller: string,
  contractId: string,
  method: string,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  args: any[],
  signTx: SignFn,
): Promise<string> {
  if (!isValidContractId(contractId)) {
    throw new ContractError(`Invalid contract ID format: ${contractId}`);
  }

  const rpc = getContractClient();
  const horizon = new Horizon.Server(HORIZON_URL);
  const account = await horizon.loadAccount(caller);

  const contract = new Contract(contractId);
  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call(method, ...args))
    .setTimeout(30)
    .build();

  // Prepare (adds footprint / resource fees)
  const prepared = await rpc.prepareTransaction(tx);
  const signedXdr = await signTx(prepared.toXDR());
  const signedTx = TransactionBuilder.fromXDR(signedXdr, NETWORK_PASSPHRASE);

  const sendResult = await rpc.sendTransaction(signedTx);
  if (sendResult.status === "ERROR") {
    throw new ContractError(
      `Submit failed: ${JSON.stringify(sendResult.errorResult)}`,
    );
  }

  // Poll until confirmed
  const hash = sendResult.hash;
  let attempts = 0;
  while (attempts < 20) {
    await new Promise((r) => setTimeout(r, 1500));
    const status = await rpc.getTransaction(hash);
    if (status.status === SorobanRpc.Api.GetTransactionStatus.SUCCESS) {
      cacheInvalidateLive(contractId);
      return hash;
    }
    if (status.status === SorobanRpc.Api.GetTransactionStatus.FAILED) {
      throw new ContractError(`Transaction failed on-chain: ${hash}`);
    }
    attempts++;
  }
  throw new ContractError(`Transaction not confirmed after polling: ${hash}`);
}

// ── Public API ────────────────────────────────────────────────────────────────

/**
 * Fetches high-level campaign metadata from the contract.
 * @param {string} [contractId=CONTRACT_ID] - The Soroban contract address
 * @returns {Promise<CampaignInfo>} Decoded campaign metadata
 * @throws {ContractError} If the contract call fails
 */
export async function getCampaignInfo(
  contractId: string = CONTRACT_ID,
): Promise<CampaignInfo> {
  const [
    title,
    description,
    creator,
    goal,
    deadline,
    minContribution,
    maxContribution,
  ] = await Promise.all([
    simulateView(contractId, "title"),
    simulateView(contractId, "description"),
    simulateView(contractId, "creator"),
    simulateView(contractId, "goal"),
    simulateView(contractId, "deadline"),
    simulateView(contractId, "min_contribution"),
    simulateView(contractId, "max_contribution"),
  ]);
  return {
    title: String(title),
    description: String(description),
    creator: String(creator),
    goal: BigInt(goal as string | number),
    deadline: BigInt(deadline as string | number),
    minContribution: BigInt(minContribution as string | number),
    maxContribution: BigInt(maxContribution as string | number),
  };
}

/**
 * Fetches live campaign statistics (raised amount, progress, contributor count).
 * @param {string} [contractId=CONTRACT_ID] - The Soroban contract address
 * @returns {Promise<CampaignStats>} Decoded campaign statistics
 * @throws {ContractError} If the contract call fails
 */
export async function getCampaignStats(
  contractId: string = CONTRACT_ID,
): Promise<CampaignStats> {
  const raw = (await simulateView(contractId, "get_stats")) as {
    total_raised: string | number;
    progress_bps: string | number;
    contributor_count: string | number;
  };
  return {
    totalRaised: BigInt(raw.total_raised),
    progressPercent: Number(raw.progress_bps) / 100,
    contributorCount: Number(raw.contributor_count),
  };
}

/**
 * Submits a contribution to the campaign.
 * @param {string} contractId - The Soroban contract address
 * @param {string} contributor - The contributor's Stellar public key
 * @param {bigint} amount - Contribution amount in stroops
 * @param {SignFn} signTx - Wallet signing function (e.g. from WalletContext)
 * @returns {Promise<string>} Transaction hash on success
 * @throws {ContractError} If submission fails
 */
export async function contribute(
  contractId: string,
  contributor: string,
  amount: bigint,
  signTx: SignFn,
): Promise<string> {
  return invokeContract(
    contributor,
    contractId,
    "contribute",
    [
      new Address(contributor).toScVal(),
      nativeToScVal(amount, { type: "i128" }),
    ],
    signTx,
  );
}

/**
 * Withdraws raised funds to the campaign creator after a successful campaign.
 * @param {string} contractId - The Soroban contract address
 * @param {string} creator - The creator's Stellar public key
 * @param {SignFn} signTx - Wallet signing function
 * @returns {Promise<string>} Transaction hash on success
 * @throws {ContractError} If withdrawal fails
 */
export async function withdraw(
  contractId: string,
  creator: string,
  signTx: SignFn,
): Promise<string> {
  return invokeContract(creator, contractId, "withdraw", [], signTx);
}

/**
 * Claims a refund for a single contributor after a failed campaign.
 * @param {string} contractId - The Soroban contract address
 * @param {string} contributor - The contributor's Stellar public key
 * @param {SignFn} signTx - Wallet signing function
 * @returns {Promise<string>} Transaction hash on success
 * @throws {ContractError} If refund fails
 */
export async function refundSingle(
  contractId: string,
  contributor: string,
  signTx: SignFn,
): Promise<string> {
  return invokeContract(
    contributor,
    contractId,
    "refund_single",
    [new Address(contributor).toScVal()],
    signTx,
  );
}

/**
 * Refunds multiple contributors in a single transaction (batch refund).
 * The contract caps the batch at 25 contributors per call.
 * @param {string} contractId - The Soroban contract address
 * @param {string} caller - The caller's Stellar public key (any authorized address)
 * @param {string[]} contributors - List of contributor addresses to refund
 * @param {SignFn} signTx - Wallet signing function
 * @returns {Promise<string>} Transaction hash on success
 * @throws {ContractError} If batch refund fails
 */
export async function refundBatch(
  contractId: string,
  caller: string,
  contributors: string[],
  signTx: SignFn,
): Promise<string> {
  const { xdr } = await import("@stellar/stellar-sdk");
  const contributorVec = xdr.ScVal.scvVec(
    contributors.map((addr) => new Address(addr).toScVal()),
  );
  return invokeContract(
    caller,
    contractId,
    "refund_batch",
    [contributorVec],
    signTx,
  );
}

/**
 * Pauses the campaign, blocking new contributions. Admin only.
 * @param {string} contractId - The Soroban contract address
 * @param {string} admin - The admin's Stellar public key
 * @param {SignFn} signTx - Wallet signing function
 * @returns {Promise<string>} Transaction hash on success
 * @throws {ContractError} If pause fails
 */
export async function pauseCampaign(
  contractId: string,
  admin: string,
  signTx: SignFn,
): Promise<string> {
  return invokeContract(admin, contractId, "pause", [], signTx);
}

/**
 * Resumes a paused campaign, allowing contributions again. Admin only.
 * @param {string} contractId - The Soroban contract address
 * @param {string} admin - The admin's Stellar public key
 * @param {SignFn} signTx - Wallet signing function
 * @returns {Promise<string>} Transaction hash on success
 * @throws {ContractError} If unpause fails
 */
export async function unpauseCampaign(
  contractId: string,
  admin: string,
  signTx: SignFn,
): Promise<string> {
  return invokeContract(admin, contractId, "unpause", [], signTx);
}
