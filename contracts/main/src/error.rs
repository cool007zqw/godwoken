#[derive(Debug)]
#[repr(i8)]
pub enum Error {
    InvalidSince = -5,
    InvalidOutputTypeHash = -6,
    InvalidOutputLockHash = -7,
    InvalidAggregatorIndex = -8,
    InvalidChallengerIndex = -9,
    InvalidChallengeContext = -10,
    InvalidWitness = -11,
    IncorrectCapacity = -12,
    InvalidAccountCount = -110,
    InvalidAccountIndex = -13,
    InvalidAccountScript = -14,
    InvalidAccountNonce = -15,
    InvalidDepositAmount = -16,
    InvalidGlobalState = -17,
    InvalidAccountMerkleProof = -18,
    InvalidBlockMerkleProof = -19,
    InvalidAggregator = -20,
    InvalidTxRoot = -21,
    InvalidAccountRoot = -22,
    InvalidSignature = -23,
    IncorrectAgIndex = -27,
    TryRevertRevertedBlock = -33,
    InvalidNewAccountRoot = -36,
    InvalidScript = -38,
    InvalidChallengeCell = -39,
}
