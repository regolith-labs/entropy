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
/// 0.1 = 1/10 → ~92% of feeds flip per sample, with ~20x safety margin above
/// single-publisher influence.
pub const SENSITIVITY_NUM: u64 = 1;
pub const SENSITIVITY_DENOM: u64 = 10;

/// Pyth price feed tickers (matches FEED_ADDRESSES order).
pub const FEED_TICKERS: [&str; NUM_FEEDS] = [
    "BTC", "ETH", "SOL", "PENGU", "AVAX", "GOAT", "GRASS",
    "FWOG", "RAY", "ZEC", "PUMP", "HYPE", "JUP", "JTO",
    "BONK", "WIF", "PYTH", "ORCA", "MET", "KMNO", "MSOL",
    "MEW", "TRUMP", "FART", "POPCAT", "W", "XAU", "XAG",
    "EUR", "GBP", "JPY", "NZD",
];

/// Pyth price feed addresses (order must match FEED_TICKERS).
pub const FEED_ADDRESSES: [Pubkey; NUM_FEEDS] = [
    solana_program::pubkey!("4cSM2e6rvbGQUFiJbqytoVMi5GgghSMr8LwVrT9VPSPo"), // BTC/USD
    solana_program::pubkey!("42amVS4KgzR9rA28tkVYqVXjq9Qa8dcZQMbH5EYFX6XC"), // ETH/USD
    solana_program::pubkey!("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE"), // SOL/USD
    solana_program::pubkey!("27zzC5wXCeZeuJ3h9uAJzV5tGn6r5Tzo98S1ZceYKEb8"), // PENGU/USD
    solana_program::pubkey!("HUBqpBf3aGJdVQndFHmMUd1eMcixt7S4swYPCx8A93K1"), // AVAX/USD
    solana_program::pubkey!("3KebxXoZLaZvvdc3ecmdgwWQWSCLQeuouS6mrF7ar1en"), // GOAT/USD
    solana_program::pubkey!("1vdRiUwEcjRArZFYosVaPFJKyuqYrPFNvshbZ4yCACS"),  // GRASS/USD
    solana_program::pubkey!("gWzECufoh81TGMrRRD9QnTUjHQpGW1kywXu8PZYLhmF"),  // FWOG/USD
    solana_program::pubkey!("Hhipna3EoWR7u8pDruUg8RxhP5F6XLh6SEHMVDmZhWi8"), // RAY/USD
    solana_program::pubkey!("HzdKMXqocYWqy7mh8AKDoZFJinjeGMfBKmGAxGbasc28"), // ZEC/USD
    solana_program::pubkey!("HMm3GPbdnqGwbkTnUUqCFsH8AMHDdEC3Lg8gcPD3HJSH"), // PUMP/USD
    solana_program::pubkey!("6usXZCEM4kf1KHGDTzgQLAWtDMNdzLjfAUYSwGKJm19Y"), // HYPE/USD
    solana_program::pubkey!("7dbob1psH1iZBS7qPsm3Kwbf5DzSXK8Jyg31CTgTnxH5"), // JUP/USD
    solana_program::pubkey!("7ajR2zA4MGMMTqRAVjghTKqPPn4kbrj3pYkAVRVwTGzP"), // JTO/USD
    solana_program::pubkey!("DBE3N8uNjhKPRHfANdwGvCZghWXyLPdqdSbEW2XFwBiX"), // BONK/USD
    solana_program::pubkey!("6B23K3tkb51vLZA14jcEQVCA1pfHptzEHFA93V5dYwbT"), // WIF/USD
    solana_program::pubkey!("8vjchtMuJNY4oFQdTi8yCe6mhCaNBFaUbktT482TpLPS"), // PYTH/USD
    solana_program::pubkey!("4CBshVeNBEXz24GZpoj8SrqP5L7VGG3qjGd6tCST1pND"), // ORCA/USD
    solana_program::pubkey!("ApN7pa6MH2WmuZFXwi5PxMb4bSwRLDthbVSuWaiSMWyR"), // MET/USD
    solana_program::pubkey!("ArjngUHXrQPr1wH9Bqrji9hdDQirM6ijbzc1Jj1fXUk7"), // KMNO/USD
    solana_program::pubkey!("5CKzb9j4ChgLUt8Gfm5CNGLN6khXKiqMbnGAW4cgXgxK"), // MSOL/USD
    solana_program::pubkey!("EF6U755BdHMXim8RBw6XSC6Yk6XaouTKpwcBZ7QkcanB"), // MEW/USD
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
