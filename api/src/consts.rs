use steel::*;

/// Seed of the var account PDA.
pub const VAR: &[u8] = b"var";

/// The number of Pyth price feeds used for the entropy hash.
pub const NUM_FEEDS: usize = 32;

/// EWMA half-life in slots (~1 minute at 400ms/slot).
pub const HALFLIFE: u64 = 150;

/// Minimum threshold floor in basis points (1 = 0.01%).
pub const MIN_BPS: u64 = 1;

/// Sensitivity as a fraction (SENSITIVITY_NUM / SENSITIVITY_DENOM).
/// Controls how many standard deviations of price movement are required to flip a bit.
/// Lower values = more sensitive (smaller moves flip bits).
/// 0.15 = 15/100 → ~88% of feeds flip per sample, with ~29x safety margin above
/// single-publisher influence.
pub const SENSITIVITY_NUM: u64 = 15;
pub const SENSITIVITY_DENOM: u64 = 100;

/// Pyth price feed tickers (matches FEED_ADDRESSES order).
pub const FEED_TICKERS: [&str; NUM_FEEDS] = [
    "BTC", "ETH", "SOL", "BNB", "AVAX", "SUI", "APT",
    "NEAR", "UNI", "ZEC", "TRX", "HYPE", "JUP", "JTO",
    "BONK", "WIF", "PYTH", "ORCA", "MET", "KMNO", "DOGE",
    "PEPE", "TRUMP", "FART", "POPCAT", "W", "XAU", "XAG",
    "EUR", "GBP", "JPY", "NZD",
];

/// Pyth price feed addresses (order must match FEED_TICKERS).
pub const FEED_ADDRESSES: [Pubkey; NUM_FEEDS] = [
    solana_program::pubkey!("4cSM2e6rvbGQUFiJbqytoVMi5GgghSMr8LwVrT9VPSPo"), // BTC/USD
    solana_program::pubkey!("42amVS4KgzR9rA28tkVYqVXjq9Qa8dcZQMbH5EYFX6XC"), // ETH/USD
    solana_program::pubkey!("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE"), // SOL/USD
    solana_program::pubkey!("A3qp5QG9xGeJR1gexbW9b9eMMsMDLzx3rhud9SnNhwb4"), // BNB/USD
    solana_program::pubkey!("HUBqpBf3aGJdVQndFHmMUd1eMcixt7S4swYPCx8A93K1"), // AVAX/USD
    solana_program::pubkey!("GgV3a7YeVRga9prjNGEDBG9NwatSaD8rwjZ4GNjPiXTq"), // SUI/USD
    solana_program::pubkey!("9oR3Uh2zsp1CxLdsuFrg3QhY2eZ2e5eLjDgDfZ6oG2ev"), // APT/USD
    solana_program::pubkey!("4Ag6xt275tDDkdWhFsCq3vTHAvNAzKVRNiqAswzb699A"), // NEAR/USD
    solana_program::pubkey!("By6KRq5KjvEmsjumNGBXQWyedaV3sAq89yjiFm6Poy3k"), // UNI/USD
    solana_program::pubkey!("HzdKMXqocYWqy7mh8AKDoZFJinjeGMfBKmGAxGbasc28"), // ZEC/USD
    solana_program::pubkey!("k6Uy1WtqWnVHv1WNpwW8L4hmLtJCu2AqfSLLcX5kEfg"), // TRX/USD
    solana_program::pubkey!("6usXZCEM4kf1KHGDTzgQLAWtDMNdzLjfAUYSwGKJm19Y"), // HYPE/USD
    solana_program::pubkey!("7dbob1psH1iZBS7qPsm3Kwbf5DzSXK8Jyg31CTgTnxH5"), // JUP/USD
    solana_program::pubkey!("7ajR2zA4MGMMTqRAVjghTKqPPn4kbrj3pYkAVRVwTGzP"), // JTO/USD
    solana_program::pubkey!("DBE3N8uNjhKPRHfANdwGvCZghWXyLPdqdSbEW2XFwBiX"), // BONK/USD
    solana_program::pubkey!("6B23K3tkb51vLZA14jcEQVCA1pfHptzEHFA93V5dYwbT"), // WIF/USD
    solana_program::pubkey!("8vjchtMuJNY4oFQdTi8yCe6mhCaNBFaUbktT482TpLPS"), // PYTH/USD
    solana_program::pubkey!("4CBshVeNBEXz24GZpoj8SrqP5L7VGG3qjGd6tCST1pND"), // ORCA/USD
    solana_program::pubkey!("ApN7pa6MH2WmuZFXwi5PxMb4bSwRLDthbVSuWaiSMWyR"), // MET/USD
    solana_program::pubkey!("ArjngUHXrQPr1wH9Bqrji9hdDQirM6ijbzc1Jj1fXUk7"), // KMNO/USD
    solana_program::pubkey!("681QkKLoAQrB5h23Ewq9c8rjM19RBuzqwXZf2RPr9Pyw"), // DOGE/USD
    solana_program::pubkey!("3adfGDsTjqC55Mw5MfzpcLpNMKGGPBwc9M8xAYq4VEQe"), // PEPE/USD
    solana_program::pubkey!("9vNb2tQoZ8bB4vzMbQLWViGwNaDJVtct13AGgno1wazp"), // TRUMP/USD
    solana_program::pubkey!("2t8eUbYKjidMs3uSeYM9jXM9uudYZwGkSeTB4TKjmvnC"), // FARTCOIN/USD
    solana_program::pubkey!("6UxPR2nXJNNM1nESVWGAf8NXMVu3SGgYf3ZfUFoGB9cs"), // POPCAT/USD
    solana_program::pubkey!("BEMsCSQEGi2kwPA4mKnGjxnreijhMki7L4eeb96ypzF9"), // W/USD
    solana_program::pubkey!("2uPQGpm8X4ZkxMHxrAW1QuhXcse1AHEgPih6Xp9NuEWW"), // XAU/USD
    solana_program::pubkey!("H9JxsWwtDZxjSL6m7cdCVsWibj3JBMD9sxqLjadoZnot"), // XAG/USD
    solana_program::pubkey!("Fu76ChamBDjE8UuGLV6GP2AcPPSU6gjhkNhAyuoPm7ny"), // EUR/USD
    solana_program::pubkey!("G25Tm7UkVruTJ7mcbCxFm45XGWwsH72nJKNGcHEQw1tU"), // GBP/USD
    solana_program::pubkey!("AMpTDXYcq8WaDR4FG8JW239vuwzAGqeS4fJSqGZi9V2P"), // JPY/USD
    solana_program::pubkey!("A4rweVuHNya9iafJ8HhH5gP9HWnHjXvej2WD8aFhaqhc"), // NZD/USD
];
