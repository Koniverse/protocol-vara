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
} from '@subwallet/invariant-vara-sdk-wasm'
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
  '0xda99688246cb1b34a880f072257783d7dfda08be8c51a2250c05f7d138d1682a'
export const TESTNET_BTC_ADDRESS: HexString =
  '0x1ffa0525d01f0177288dd1419a66a12c20bab8db941c6831bbb03686888cb048'
export const TESTNET_ETH_ADDRESS: HexString =
  '0xcf5d1cab91d893c8a5d6686c35e46463e4957b7fa729bc73f111492d8f49b191'
export const TESTNET_USDC_ADDRESS: HexString =
  '0x8310182c5b91e4114766d877e923af825801ef3f023f8d720d52d16987f9a3c7'
export const TESTNET_SOL_ADDRESS: HexString =
  '0x6dce80d01a8ce83cbac76ddfbda67375bf3cee7ee03728a3e5946b8bd617be81'
export const TESTNET_AZERO_ADDRESS: HexString =
  '0x1735b333d1c3bd14f6c61ba6fb9c0fbb29bf530243d0d837d2c7011517096000'

