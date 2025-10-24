use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum EntropyInstruction {
    Open = 0,
    Close = 1,
    Commit = 2,
    Reveal = 4,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Open {
    pub id: [u8; 8],
    pub last_commit_at: u64,
    pub last_reveal_at: u64,
    pub close_at: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Close {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Commit {
    pub hash: [u8; 32],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Reveal {
    pub seed: [u8; 32],
}

instruction!(EntropyInstruction, Open);
instruction!(EntropyInstruction, Close);
instruction!(EntropyInstruction, Commit);
instruction!(EntropyInstruction, Reveal);
