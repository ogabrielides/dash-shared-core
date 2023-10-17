use byte::BytesExt;
use hashes::hex::ToHex;
use std::collections::BTreeMap;
use crate::chain::common::LLMQType;
use crate::chain::constants::{CORE_PROTO_20, CORE_PROTO_BLS_BASIC};
use crate::consensus::encode::VarInt;
use crate::crypto::byte_util::{BytesDecodable, Reversable};
use crate::crypto::var_array::VarArray;
use crate::crypto::UInt256;
use crate::models::{llmq_entry::LLMQEntry, masternode_entry::{MasternodeEntry, MasternodeReadContext}, quorums_cl_sigs_object::QuorumsCLSigsObject};
use crate::tx::coinbase_transaction::CoinbaseTransaction;

#[derive(Clone)]
pub struct MNListDiff {
    pub base_block_hash: UInt256,
    pub block_hash: UInt256,
    pub total_transactions: u32,
    pub merkle_hashes: Vec<UInt256>,
    pub merkle_flags: Vec<u8>,
    pub coinbase_transaction: CoinbaseTransaction,
    pub deleted_masternode_hashes: Vec<UInt256>,
    pub added_or_modified_masternodes: BTreeMap<UInt256, MasternodeEntry>,
    pub deleted_quorums: BTreeMap<LLMQType, Vec<UInt256>>,
    pub added_quorums: BTreeMap<LLMQType, BTreeMap<UInt256, LLMQEntry>>,
    pub base_block_height: u32,
    pub block_height: u32,
    // 0: protocol_version < 70225
    // 1: all pubKeyOperator of all CSimplifiedMNListEntry are serialised using legacy BLS scheme
    // 2: all pubKeyOperator of all CSimplifiedMNListEntry are serialised using basic BLS scheme
    pub version: u16,

    // protocol_version > 70228
    // 19.2 goes with 70228
    // 20.0 goes with 70228+?
    pub quorums_cls_sigs: Vec<QuorumsCLSigsObject>,
}

impl std::fmt::Debug for MNListDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MNListDiff")
            .field("base_block_hash", &self.base_block_hash)
            .field("block_hash", &self.block_hash)
            .field("total_transactions", &self.total_transactions)
            .field("merkle_hashes", &self.merkle_hashes)
            .field("merkle_flags", &self.merkle_flags.to_hex())
            .field("merkle_flags_count", &self.merkle_flags.len())
            .field("coinbase_transaction", &self.coinbase_transaction)
            .field("deleted_masternode_hashes", &self.deleted_masternode_hashes)
            .field("added_or_modified_masternodes", &self.added_or_modified_masternodes)
            .field("deleted_quorums", &self.deleted_quorums)
            .field("added_quorums", &self.added_quorums)
            .field("base_block_height", &self.base_block_height)
            .field("block_height", &self.block_height)
            .field("version", &self.version)
            .finish()
    }
}

// impl<V> consensus::Decodable for MNListDiff {
//     #[inline]
//     fn consensus_decode<D: io::Read>(mut d: D) -> Result<Self, encode::Error> {
//         let base_block_hash = UInt256::consensus_decode(&mut d)?;
//         let block_hash = UInt256::consensus_decode(&mut d)?;
//         let total_transactions = u32::consensus_decode(&mut d)?;
//         let merkle_hashes: Vec<UInt256> = Vec::consensus_decode(&mut d)?;
//         let merkle_flags: Vec<u8> = Vec::consensus_decode(&mut d)?;
//         let coinbase_transaction = CoinbaseTransaction::consensus_decode(&mut d)?;
//
//         let version = if protocol_version >= CORE_PROTO_BLS_BASIC {
//             // BLS Basic
//             u16::consensus_decode(&mut d)?
//         } else {
//             // BLS Legacy
//             1
//         };
//         let deleted_masternode_hashes: Vec<UInt256> = Vec::consensus_decode(&mut d)?;
//         let added_or_modified_masternodes: Vec<MasternodeEntry> = Vec::consensus_decode(&mut d)?;
//
//         let added_or_modified_masternodes: BTreeMap<UInt256, MasternodeEntry> = added_or_modified_masternodes
//             .iter()
//             .fold(BTreeMap::new(), |mut acc, _| {
//                 if let Ok(entry) = message.read_with::<MasternodeEntry>(offset, masternode_read_ctx) {
//                     acc.insert(entry.provider_registration_transaction_hash.reversed(), entry);
//                 }
//                 acc
//             });
//     }
// }

impl MNListDiff {
    pub fn new<F: Fn(UInt256) -> u32>(
        protocol_version: u32,
        message: &[u8],
        offset: &mut usize,
        block_height_lookup: F,
    ) -> Result<Self, byte::Error> {
        let base_block_hash = UInt256::from_bytes(message, offset)?;
        let block_hash = UInt256::from_bytes(message, offset)?;
        let base_block_height = block_height_lookup(base_block_hash);
        let block_height = block_height_lookup(block_hash);
        let total_transactions = u32::from_bytes(message, offset)?;
        let merkle_hashes = VarArray::<UInt256>::from_bytes(message, offset)?;
        let merkle_flags_count = VarInt::from_bytes(message, offset)?.0 as usize;
        let merkle_flags: &[u8] = message.read_with(offset, byte::ctx::Bytes::Len(merkle_flags_count))?;
        let coinbase_transaction = CoinbaseTransaction::from_bytes(message, offset)?;
        let version = if protocol_version >= CORE_PROTO_BLS_BASIC {
            // BLS Basic
            u16::from_bytes(message, offset)?
        } else {
            // BLS Legacy
            1
        };
        let deleted_masternode_count = VarInt::from_bytes(message, offset)?.0;
        let mut deleted_masternode_hashes: Vec<UInt256> =
            Vec::with_capacity(deleted_masternode_count as usize);
        for _i in 0..deleted_masternode_count {
            deleted_masternode_hashes.push(UInt256::from_bytes(message, offset)?);
        }
        let added_masternode_count = VarInt::from_bytes(message, offset)?.0;
        let masternode_read_ctx = MasternodeReadContext(block_height, version, protocol_version);
        let added_or_modified_masternodes: BTreeMap<UInt256, MasternodeEntry> = (0..added_masternode_count)
            .fold(BTreeMap::new(), |mut acc, _| {
                if let Ok(entry) = message.read_with::<MasternodeEntry>(offset, masternode_read_ctx) {
                    acc.insert(entry.provider_registration_transaction_hash.reversed(), entry);
                }
                acc
            });

        let mut deleted_quorums: BTreeMap<LLMQType, Vec<UInt256>> = BTreeMap::new();
        let mut added_quorums: BTreeMap<LLMQType, BTreeMap<UInt256, LLMQEntry>> = BTreeMap::new();
        let quorums_active = coinbase_transaction.coinbase_transaction_version >= 2;
        if quorums_active {
            let deleted_quorums_count = VarInt::from_bytes(message, offset)?.0;
            for _i in 0..deleted_quorums_count {
                let llmq_type = LLMQType::from_bytes(message, offset)?;
                let llmq_hash = UInt256::from_bytes(message, offset)?;
                deleted_quorums
                    .entry(llmq_type)
                    .or_insert_with(Vec::new)
                    .push(llmq_hash);
            }
            let added_quorums_count = VarInt::from_bytes(message, offset)?.0;
            for _i in 0..added_quorums_count {
                if let Ok(entry) = LLMQEntry::from_bytes(message, offset) {
                    added_quorums
                        .entry(entry.llmq_type)
                        .or_insert_with(BTreeMap::new)
                        .insert(entry.llmq_hash, entry);
                }
            }
        }
        let mut quorums_cls_sigs = Vec::new();
        if protocol_version >= CORE_PROTO_20 { // Core v0.20
            let quorums_cl_sigs_count = VarInt::from_bytes(message, offset)?.0;
            for _i in 0..quorums_cl_sigs_count {
                let sig = QuorumsCLSigsObject::from_bytes(message, offset)?;
                quorums_cls_sigs.push(sig);
            }
        }

        Ok(Self {
            base_block_hash,
            block_hash,
            total_transactions,
            merkle_hashes: merkle_hashes.1,
            merkle_flags: merkle_flags.to_vec(),
            coinbase_transaction,
            deleted_masternode_hashes,
            added_or_modified_masternodes,
            deleted_quorums,
            added_quorums,
            base_block_height,
            block_height,
            version,
            quorums_cls_sigs
        })
    }

    pub fn has_basic_scheme_keys(&self) -> bool {
        self.added_or_modified_masternodes.values().any(|m| m.operator_public_key.version == 2)
    }

    pub fn should_skip_removed_masternodes(&self) -> bool {
        self.version >= 2
    }
}
