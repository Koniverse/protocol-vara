import {
  getGlobalMaxSqrtPrice,
  getGlobalMinSqrtPrice,
  getMaxSwapStep,
  getChunkSize,
  getLiquidityTickLimit,
  getMaxPoolKeysReturned,
  getPositionEntriesLimit,
  getMaxPoolPairsReturned,
  getPriceScale,
  getPercentageScale,
  getLiquidityScale,
  getFixedPointScale,
  getFeeGrowthScale,
  getSecondsPerLiquidityScale,
  getSqrtPriceScale,
  getTokenAmountScale,
  getFeeGrowthDenominator,
  getFixedPointDenominator,
  getLiquidityDenominator,
  getPercentageDenominator,
  getPriceDenominator,
  getSecondsPerLiquidityDenominator,
  getSqrtPriceDenominator,
  getTokenAmountDenominator,
  getTickSearchRange
} from '@invariant-labs/vara-sdk-wasm'
import { HexString } from './utils.js'

export const LOCAL = 'ws://127.0.0.1:9944'
export const TESTNET = 'wss://testnet.vara.network'
export const MAINNET = 'wss://rpc.vara.network'

export const FUNGIBLE_TOKEN_GAS_LIMIT = 750_000_000_000n
export const INVARIANT_GAS_LIMIT = 750_000_000_000n
export const DEFAULT_ADDRESS = '5F3sa2TJAWMqDhXG6jhV4N8ko9SxwGy8TpaNS1repo5EYjQX'

export const MAX_SQRT_PRICE = getGlobalMaxSqrtPrice()
export const MIN_SQRT_PRICE = getGlobalMinSqrtPrice()
export const MAX_SWAP_STEPS = getMaxSwapStep()
export const SEARCH_RANGE = getTickSearchRange()
export const CHUNK_SIZE = getChunkSize()
export const LIQUIDITY_TICKS_LIMIT = getLiquidityTickLimit()
export const MAX_POOL_PAIRS_RETURNED = getMaxPoolPairsReturned()
export const MAX_POOL_KEYS_RETURNED = getMaxPoolKeysReturned()
export const POSITIONS_ENTRIES_LIMIT = getPositionEntriesLimit()

export const FEE_GROWTH_DENOMINATOR = getFeeGrowthDenominator()
export const FIXED_POINT_DENOMINATOR = getFixedPointDenominator()
export const LIQUIDITY_DENOMINATOR = getLiquidityDenominator()
export const PERCENTAGE_DENOMINATOR = getPercentageDenominator()
export const PRICE_DENOMINATOR = getPriceDenominator()
export const SECONDS_PER_LIQUIDITY_DENOMINATOR = getSecondsPerLiquidityDenominator()
export const SQRT_PRICE_DENOMINATOR = getSqrtPriceDenominator()
export const TOKEN_AMOUNT_DENOMINATOR = getTokenAmountDenominator()

export const FEE_GROWTH_SCALE = getFeeGrowthScale()
export const FIXED_POINT_SCALE = getFixedPointScale()
export const LIQUIDITY_SCALE = getLiquidityScale()
export const PERCENTAGE_SCALE = getPercentageScale()
export const PRICE_SCALE = getPriceScale()
export const SECONDS_PER_LIQUIDITY_SCALE = getSecondsPerLiquidityScale()
export const SQRT_PRICE_SCALE = getSqrtPriceScale()
export const TOKEN_AMOUNT_SCALE = getTokenAmountScale()

export const CONCENTRATION_FACTOR = 1.00001526069123

export const VARA_ADDRESS: HexString =
  '0x0000000000000000000000000000000000000000000000000000000000000000'
export const TESTNET_INVARIANT_ADDRESS: HexString =
  '0x06c71389bc9370b8e6a2d35b676cea3973c8ab10187e8390945ebb01fb6b66cd'
export const TESTNET_BTC_ADDRESS: HexString =
  '0x80ea6ace20dcbfc95274987b4031ccc3e416461e9980b2e397eba4b5b470ff03'
export const TESTNET_ETH_ADDRESS: HexString =
  '0x59be95ad63df8369acc820d07bee8f86c75cbe72db3bdcb6822c0e7f3e2c8f50'
export const TESTNET_USDC_ADDRESS: HexString =
  '0x5e5dbe4219abd39812b9b9dd80f2f3bf47c497a1910e64802abf97ca5edd4415'
export const TESTNET_SOL_ADDRESS: HexString =
  '0x0d3b71c84950eb9d33a55af4a52523fffdb821601694653e7d038fb70de7fed6'
export const TESTNET_AZERO_ADDRESS: HexString =
  '0x7d25efa0ec2eb985367f359bd8a213f10aad78cfa0c020f7642f8393cc69d773'

