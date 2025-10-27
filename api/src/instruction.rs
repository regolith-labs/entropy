use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum EntropyInstruction {
    Open = 0,
    Close = 1,
    Next = 2,
    Reveal = 4,
    Sample = 5,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Open {
    /// The commit provided by Entropy provider.
    pub commit: [u8; 32],

    /// Whether or not the Entropy provider should automatically sample the slot hash.
    pub is_auto: u64,

    /// The number of random variables to sample.
    pub samples: u64,

    /// The slot at which the variable should sample the slothash.
    pub end_at: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Close {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Next {
    pub end_at: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Reveal {
    pub seed: [u8; 32],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Sample {}

instruction!(EntropyInstruction, Open);
instruction!(EntropyInstruction, Close);
instruction!(EntropyInstruction, Next);
instruction!(EntropyInstruction, Reveal);
instruction!(EntropyInstruction, Sample);
