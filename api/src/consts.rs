use steel::*;

/// Seed of the var account PDA.
pub const VAR: &[u8] = b"var";

/// The number of Pyth price feeds used for the entropy hash.
pub const NUM_FEEDS: usize = 32;

/// EWMA half-life in slots (~1 minute at 400ms/slot).
pub const HALFLIFE: u64 = 150;

/// Minimum threshold floor in basis points (1 = 0.01%).
pub const MIN_BPS: u64 = 1;


/// Pyth price feed tickers (matches FEED_ADDRESSES order).
pub const FEED_TICKERS: [&str; NUM_FEEDS] = [
    "BTC", "SOL", "ETH", "JUP", "JTO", "DOGE", "COIN",
    "AAPL", "SPY", "TSLA", "NVDA", "GOOGL", "MSFT", "META",
    "AMZN", "XAU", "XAG", "EUR", "GBP", "JPY", "AUD",
    "MSTR", "PLTR", "HOOD", "RKLB", "CRWD", "LLY", "AVGO",
    "BONK", "WIF", "RNDR", "SUI",
];

/// Pyth price feed addresses (order must match FEED_TICKERS).
pub const FEED_ADDRESSES: [Pubkey; NUM_FEEDS] = [
    solana_program::pubkey!("4cSM2e6rvbGQUFiJbqytoVMi5GgghSMr8LwVrT9VPSPo"), // BTC/USD
    solana_program::pubkey!("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE"), // SOL/USD
    solana_program::pubkey!("42amVS4KgzR9rA28tkVYqVXjq9Qa8dcZQMbH5EYFX6XC"), // ETH/USD
    solana_program::pubkey!("7dbob1psH1iZBS7qPsm3Kwbf5DzSXK8Jyg31CTgTnxH5"), // JUP/USD
    solana_program::pubkey!("7ajR2zA4MGMMTqRAVjghTKqPPn4kbrj3pYkAVRVwTGzP"), // JTO/USD
    solana_program::pubkey!("681QkKLoAQrB5h23Ewq9c8rjM19RBuzqwXZf2RPr9Pyw"), // DOGE/USD
    solana_program::pubkey!("91JXaWGHr57awfqhXQP2TxrkLX6CpvtBaaRjz1PEQqXn"), // COIN/USD
    solana_program::pubkey!("DJ2FyTgUAkEtXW3U5P9PF19meFTRtW4ZWKKFgACfVbUy"), // AAPL/USD
    solana_program::pubkey!("9owhtgrdLiUMAH9JKxYFt5pUY4Luy4EzzLhdcWPVuDyy"), // SPY/USD
    solana_program::pubkey!("E8WFH8brgP58arcuW2wwsPHiomYrSvrgWTsRLZLAEZUQ"), // TSLA/USD
    solana_program::pubkey!("2w1Tg1XTZbUib7srfRoStJ4v5JXVsK7roQEGMsMaGZFC"), // NVDA/USD
    solana_program::pubkey!("HShKFQqhYkUiXpVyyLmrAALXwWqHB7ikLmPbrwJzpRNh"), // GOOGL/USD
    solana_program::pubkey!("7VYuuJxz8w2rLA9tJG2KZ9T1fSMcjC7uECoYA6nDaqtK"), // MSFT/USD
    solana_program::pubkey!("GsKrMNoa1Mqjpif4SYk2WjdduWZP699hXRdP51yBM6K2"), // META/USD
    solana_program::pubkey!("GBkjjFxbaFY9TBHpAPypk5JBchpPPve2jskAcd9zuFNd"), // AMZN/USD
    solana_program::pubkey!("2uPQGpm8X4ZkxMHxrAW1QuhXcse1AHEgPih6Xp9NuEWW"), // XAU/USD
    solana_program::pubkey!("H9JxsWwtDZxjSL6m7cdCVsWibj3JBMD9sxqLjadoZnot"), // XAG/USD
    solana_program::pubkey!("Fu76ChamBDjE8UuGLV6GP2AcPPSU6gjhkNhAyuoPm7ny"), // EUR/USD
    solana_program::pubkey!("G25Tm7UkVruTJ7mcbCxFm45XGWwsH72nJKNGcHEQw1tU"), // GBP/USD
    solana_program::pubkey!("AMpTDXYcq8WaDR4FG8JW239vuwzAGqeS4fJSqGZi9V2P"), // USD/JPY
    solana_program::pubkey!("6pPXqXcgFFoLEcXfedWJy3ypNZVJ1F3mgipaDFsvZ1co"), // AUD/USD
    solana_program::pubkey!("HJGvGyWrAXdZPG4Q7LNkkKja72FDkJW7ixuyg3u6vZyP"), // MSTR/USD
    solana_program::pubkey!("7RP45Z6dsTrHQakMg7xha1RLZGk1x2pVViBjpUMpzdBK"), // PLTR/USD
    solana_program::pubkey!("5tZizzQN776ZWTibPJKecjk1DkTSDHu47dXM3SxR5D5i"), // HOOD/USD
    solana_program::pubkey!("H2tjxYMHGVN9F8S7ewVaECDZtRpVxgfrtAMEGtRDvqYe"), // RKLB/USD
    solana_program::pubkey!("8zWQVp313FFdanpZoQeDohp5HE7ugoJE2VaX4sYPHj4e"), // CRWD/USD
    solana_program::pubkey!("AmhgzXb37V3YegqdXoDTGL5QVhSV83dESyadboJwc7sQ"), // LLY/USD
    solana_program::pubkey!("2jgfs5FsDQkdCrgcCKHEd7p9KNtKAyWznMSyu21WbFgS"), // AVGO/USD
    // TODO: Verify these 4 Pyth pull oracle addresses on mainnet before deploying
    solana_program::pubkey!("DBE3N8uNjhKPRHfANdwGvCZghWXyLPdqdSbEW2XFwBiX"), // BONK/USD
    solana_program::pubkey!("6ABgrEZk8urs6kJ1JNdC1sspH5zKXRqxy8sg3ZG2cQps"), // WIF/USD
    solana_program::pubkey!("AnLf8tVYCM816gmBjiy8n53eXKKEDydT5piYjjQDPgTB"), // RNDR/USD
    solana_program::pubkey!("3Qub3HaAJaa2xNY7SUqPKd3vVwTqDfDDkEUMPjXD2c1q"), // SUI/USD
];
