use std::collections::BTreeMap;
use byte::BytesExt;
use rs_ffi_interfaces::boxed;
use crate::{common, models, ok_or_return_processing_error, processing, types};
use crate::chain::common::{IHaveChainSettings, LLMQType, LLMQParams};
use crate::consensus::encode;
use crate::crypto::{byte_util::{Reversable, Zeroable}, UInt256};
use crate::crypto::byte_util::BytesDecodable;
use crate::ffi::to::ToFFI;
use crate::processing::{CoreProvider, MasternodeProcessorCache, MNListDiffResult, ProcessingError};

// https://github.com/rust-lang/rfcs/issues/2770
#[repr(C)]
#[derive(Debug)]
pub struct MasternodeProcessor {
    pub provider: Box<dyn CoreProvider>,
}

impl MasternodeProcessor {
    pub fn new<T: CoreProvider + 'static>(provider: T) -> Self {
        Self { provider: Box::new(provider) }
    }
}

impl MasternodeProcessor {

    pub fn get_list_diff_result_with_base_lookup(
        &self,
        list_diff: models::MNListDiff,
        should_process_quorums: bool,
        is_dip_0024: bool,
        is_rotated_quorums_presented: bool,
        cache: &mut MasternodeProcessorCache,
    ) -> types::MNListDiffResult {
        let base_block_hash = list_diff.base_block_hash;
        let base_list = self.provider.find_masternode_list(
            base_block_hash,
            &cache.mn_lists,
            &mut cache.needed_masternode_lists,
        );
        self.get_list_diff_result(base_list.ok(), list_diff, should_process_quorums, is_dip_0024, is_rotated_quorums_presented, cache)
    }

    pub fn get_list_diff_result_internal_with_base_lookup(
        &self,
        list_diff: models::MNListDiff,
        should_process_quorums: bool,
        is_dip_0024: bool,
        is_rotated_quorums_presented: bool,
        cache: &mut MasternodeProcessorCache,
    ) -> MNListDiffResult {
        let base_list = self.provider.find_masternode_list(
            list_diff.base_block_hash,
            &cache.mn_lists,
            &mut cache.needed_masternode_lists,
        );
        self.get_list_diff_result_internal(base_list.ok(), list_diff, should_process_quorums, is_dip_0024, is_rotated_quorums_presented, cache)
    }

    pub(crate) fn get_list_diff_result(
        &self,
        base_list: Option<models::MasternodeList>,
        list_diff: models::MNListDiff,
        should_process_quorums: bool,
        is_dip_0024: bool,
        is_rotated_quorums_presented: bool,
        cache: &mut MasternodeProcessorCache,
    ) -> types::MNListDiffResult {
        let result = self.get_list_diff_result_internal(base_list, list_diff, should_process_quorums, is_dip_0024, is_rotated_quorums_presented, cache);
        // println!("get_list_diff_result: {:#?}", result);
        result.encode()
    }

    fn cache_masternode_list(
        &self,
        block_hash: UInt256,
        list: models::MasternodeList,
        cache: &mut MasternodeProcessorCache,
    ) {
        // It's good to cache lists to use it inside processing session
        // Here we use opaque-like pointer which we initiate on the C-side to sync its lifetime with runtime
        #[cfg(feature = "generate-dashj-tests")]
        crate::util::java::save_masternode_list_to_json(&list, self.lookup_block_height_by_hash(block_hash));
        cache.add_masternode_list(block_hash, list);
        // Here we just store it in the C-side ()
        // self.save_masternode_list(block_hash, &masternode_list);
    }

    pub(crate) fn get_list_diff_result_internal(
        &self,
        base_list: Option<models::MasternodeList>,
        list_diff: models::MNListDiff,
        should_process_quorums: bool,
        is_dip_0024: bool,
        is_rotated_quorums_presented: bool,
        cache: &mut MasternodeProcessorCache,
    ) -> MNListDiffResult {
        let skip_removed_masternodes = list_diff.should_skip_removed_masternodes();
        let base_block_hash = list_diff.base_block_hash;
        let block_hash = list_diff.block_hash;
        let block_height = list_diff.block_height;
        let (base_masternodes, base_quorums) = match base_list {
            Some(list) => (list.masternodes, list.quorums),
            None => (BTreeMap::new(), BTreeMap::new()),
        };
        let mut coinbase_transaction = list_diff.coinbase_transaction;
        let quorums_active = coinbase_transaction.coinbase_transaction_version >= 2;
        let (added_masternodes, modified_masternodes, masternodes) = self.classify_masternodes(
            base_masternodes,
            list_diff.added_or_modified_masternodes,
            list_diff.deleted_masternode_hashes,
            block_height,
            block_hash,
        );
        let (added_quorums, quorums, has_valid_quorums) = self.classify_quorums(
            base_quorums,
            list_diff.added_quorums,
            list_diff.deleted_quorums,
            should_process_quorums,
            skip_removed_masternodes,
            is_dip_0024,
            is_rotated_quorums_presented,
            cache,
        );
        let masternode_list = models::MasternodeList::new(
            masternodes,
            quorums,
            block_hash,
            block_height,
            quorums_active,
        );
        let merkle_tree = common::MerkleTree {
            tree_element_count: list_diff.total_transactions,
            hashes: list_diff.merkle_hashes,
            flags: list_diff.merkle_flags.as_slice(),
        };
        self.cache_masternode_list(block_hash, masternode_list.clone(), cache);
        let needed_masternode_lists = cache.needed_masternode_lists.clone();
        cache.needed_masternode_lists.clear();
        let has_found_coinbase = coinbase_transaction.has_found_coinbase(&merkle_tree.hashes);
        let desired_merkle_root = self.provider.lookup_merkle_root_by_hash(block_hash).unwrap_or(UInt256::MIN);
        let has_valid_coinbase = merkle_tree.has_root(desired_merkle_root);
        let has_valid_mn_list_root = masternode_list.has_valid_mn_list_root(&coinbase_transaction);
        let has_valid_llmq_list_root = !quorums_active || masternode_list.has_valid_llmq_list_root(&coinbase_transaction);
        let result = MNListDiffResult {
            // error_status: ProcessingError::None,
            base_block_hash,
            block_hash,
            has_found_coinbase,
            has_valid_coinbase,
            has_valid_mn_list_root,
            has_valid_llmq_list_root,
            has_valid_quorums,
            masternode_list,
            added_masternodes,
            modified_masternodes,
            added_quorums,
            needed_masternode_lists,
            quorums_cl_sigs: list_diff.quorums_cls_sigs,
        };
        result
    }

    pub fn classify_masternodes(
        &self,
        base_masternodes: BTreeMap<UInt256, models::MasternodeEntry>,
        added_or_modified_masternodes: BTreeMap<UInt256, models::MasternodeEntry>,
        deleted_masternode_hashes: Vec<UInt256>,
        block_height: u32,
        block_hash: UInt256,
    ) -> (
        BTreeMap<UInt256, models::MasternodeEntry>,
        BTreeMap<UInt256, models::MasternodeEntry>,
        BTreeMap<UInt256, models::MasternodeEntry>,
    ) {
        let added_masternodes = added_or_modified_masternodes
            .iter()
            .filter(|(k, _)| !base_masternodes.contains_key(k))
            .map(|(k, v)| (*k, v.clone()))
            .collect::<BTreeMap<_, _>>();

        let mut modified_masternodes = added_or_modified_masternodes
            .iter()
            .filter(|(k, _)| base_masternodes.contains_key(k))
            .map(|(k, v)| (*k, v.clone()))
            .collect::<BTreeMap<_, _>>();

        let mut masternodes = if !base_masternodes.is_empty() {
            let mut old_masternodes = base_masternodes;
            for hash in deleted_masternode_hashes {
                old_masternodes.remove(&hash.reversed());
            }
            old_masternodes.extend(added_masternodes.clone());
            old_masternodes
        } else {
            added_masternodes.clone()
        };

        for (hash, modified) in &mut modified_masternodes {
            if let Some(old) = masternodes.get_mut(hash) {
                if old.update_height < modified.update_height {
                    modified.update_with_previous_entry(old, block_height, block_hash);
                    if !old.confirmed_hash.is_zero() &&
                        old.known_confirmed_at_height.is_some() &&
                        old.known_confirmed_at_height.unwrap() > block_height {
                        old.known_confirmed_at_height = Some(block_height);
                    }
                }
                masternodes.insert(*hash, modified.clone());
            }
        }
        (added_masternodes, modified_masternodes, masternodes)
    }

    #[allow(clippy::type_complexity)]
    pub fn classify_quorums(
        &self,
        mut base_quorums: BTreeMap<LLMQType, BTreeMap<UInt256, models::LLMQEntry>>,
        mut added_quorums: BTreeMap<LLMQType, BTreeMap<UInt256, models::LLMQEntry>>,
        deleted_quorums: BTreeMap<LLMQType, Vec<UInt256>>,
        should_process_quorums: bool,
        skip_removed_masternodes: bool,
        is_dip_0024: bool,
        is_rotated_quorums_presented: bool,
        cache: &mut MasternodeProcessorCache,
    ) -> (
        BTreeMap<LLMQType, BTreeMap<UInt256, models::LLMQEntry>>,
        BTreeMap<LLMQType, BTreeMap<UInt256, models::LLMQEntry>>,
        bool,
    ) {
        let mut has_valid_quorums = true;
        if should_process_quorums {
            for (&llmq_type, llmqs_of_type) in &mut added_quorums {
                if self.should_process_quorum(llmq_type, is_dip_0024, is_rotated_quorums_presented) {
                    for (&llmq_block_hash, quorum) in llmqs_of_type {
                        has_valid_quorums &= self.validate_quorum(quorum, skip_removed_masternodes, llmq_block_hash, cache);
                    }
                }
            }
        }
        for (llmq_type, keys_to_delete) in &deleted_quorums {
            if let Some(llmq_map) = base_quorums.get_mut(llmq_type) {
                for key in keys_to_delete {
                    llmq_map.remove(key);
                }
            }
        }
        for (llmq_type, keys_to_add) in added_quorums.iter() {
            base_quorums
                .entry(*llmq_type)
                .or_insert_with(BTreeMap::new)
                .extend(keys_to_add.clone());
        }
        (added_quorums, base_quorums, has_valid_quorums)
    }

    pub fn validate_quorum(&self, quorum: &mut models::LLMQEntry, skip_removed_masternodes: bool, llmq_block_hash: UInt256, cache: &mut MasternodeProcessorCache) -> bool {
        //println!("get list for quorum {}: find_masternode_list for", llmq_block_hash);
        if let Ok(models::MasternodeList { masternodes, .. }) = self.provider.find_masternode_list(llmq_block_hash, &cache.mn_lists, &mut cache.needed_masternode_lists) {
            let valid = self.validate_quorum_with_masternodes(quorum, skip_removed_masternodes, llmq_block_hash, masternodes, cache);
            // TMP Testnet Platform LLMQ fail verification
            // if llmq_type != LLMQType::Llmqtype25_67 {
                return valid;
            // }
        }
        true
    }

    pub fn validate_quorum_with_masternodes(
        &self,
        quorum: &mut models::LLMQEntry,
        skip_removed_masternodes: bool,
        block_hash: UInt256,
        masternodes: BTreeMap<UInt256, models::MasternodeEntry>,
        cache: &mut MasternodeProcessorCache,
    ) -> bool {
        let block_height = self.provider.lookup_block_height_by_hash(block_hash);
        let valid_masternodes = if quorum.index.is_some() {
            self.get_rotated_masternodes_for_quorum(
                quorum.llmq_type,
                block_hash,
                block_height,
                &mut cache.llmq_members,
                &mut cache.llmq_indexed_members,
                &cache.mn_lists,
                &cache.llmq_snapshots,
                &mut cache.needed_masternode_lists,
                skip_removed_masternodes,
            )
        } else {
            models::MasternodeList::get_masternodes_for_quorum(
                quorum.llmq_type,
                masternodes,
                quorum.llmq_quorum_hash(),
                block_height,
                quorum.llmq_type == self.provider.chain_type().platform_type() && !quorum.version.use_bls_legacy()
            )
        };
        //crate::util::java::generate_final_commitment_test_file(self.chain_type, block_height, &quorum, &valid_masternodes);
        quorum.verify(valid_masternodes, block_height)
    }

    fn sort_scored_masternodes(scored_masternodes: BTreeMap<UInt256, models::MasternodeEntry>) -> Vec<models::MasternodeEntry> {
        // let mut v: Vec<_> = scored_masternodes.clone().into_iter().collect::<Vec<_>>();
        // v.sort_unstable_by_key(|(s, _)| std::cmp::Reverse(s.reversed()));
        // v.into_iter().map(|(_, node)| node).collect()
        let mut v = Vec::from_iter(scored_masternodes);
        v.sort_by(|(s1, _), (s2, _)| s2.reversed().cmp(&s1.reversed()));
        v.into_iter().map(|(s, node)| node).collect()
    }

    pub fn valid_masternodes_for_rotated_quorum_map(
        masternodes: Vec<models::MasternodeEntry>,
        quorum_modifier: UInt256,
        block_height: u32,
    ) -> Vec<models::MasternodeEntry> {
        let scored_masternodes = masternodes
            .into_iter()
            .filter_map(|entry| models::MasternodeList::masternode_score(&entry, quorum_modifier, block_height)
                .map(|score| (score, entry)))
            .collect::<BTreeMap<_, _>>();
        Self::sort_scored_masternodes(scored_masternodes)
    }

    // Reconstruct quorum members at index from snapshot
    pub fn quorum_quarter_members_by_snapshot(
        &self,
        llmq_params: &LLMQParams,
        quorum_base_block_height: u32,
        cached_lists: &BTreeMap<UInt256, models::MasternodeList>,
        cached_snapshots: &BTreeMap<UInt256, models::LLMQSnapshot>,
        unknown_lists: &mut Vec<UInt256>,
    ) -> Vec<Vec<models::MasternodeEntry>> {
        let work_block_height = quorum_base_block_height - 8;
        let llmq_type = llmq_params.r#type;
        let quorum_count = llmq_params.signing_active_quorum_count as usize;
        let quorum_size = llmq_params.size;
        let quarter_size = (quorum_size / 4) as usize;
        // Quorum members dichotomy in snapshot
        match self.provider.masternode_info_for_height(work_block_height, cached_lists, cached_snapshots, unknown_lists) {
            Ok((masternode_list, snapshot, work_block_hash)) => {
                let mut i: u32 = 0;
                // println!("•••• quorum_quarter_members_by_snapshot: {:?}: {:?}: {}: {}", llmq_type, snapshot.skip_list_mode, work_block_height, work_block_hash.reversed());
                // println!("{:#?}", masternode_list);
                // println!("••••");
                // java::generate_snapshot(&snapshot, work_block_height);
                // java::generate_llmq_hash(llmq_type, work_block_hash.reversed());
                // java::generate_masternode_list_from_map(&masternode_list.masternodes);
                let quorum_modifier = models::LLMQEntry::build_llmq_quorum_hash(llmq_type, work_block_hash);
                // println!("quorum_modifier: {}", quorum_modifier);
                // println!("snapshot: {:?}", snapshot);
                let scored_masternodes = models::MasternodeList::score_masternodes_map(masternode_list.masternodes, quorum_modifier, work_block_height, false);
                // java::generate_masternode_list_from_map(&scored_masternodes);
                let sorted_scored_masternodes = Self::sort_scored_masternodes(scored_masternodes);
                // println!("//////////////////sorted_scored_masternodes////////////////////");
                // println!("{:#?}", sorted_scored_masternodes.iter().map(|n| n.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>());
                let (used_at_h, unused_at_h) = sorted_scored_masternodes
                    .into_iter()
                    .partition(|_| {
                        let is_true = snapshot.member_is_true_at_index(i);
                        i += 1;
                        is_true
                    });
                // java::generate_masternode_list(&used_at_h);
                // java::generate_masternode_list(&unused_at_h);
                // println!("//////////////////////////////////////");
                let sorted_used_at_h = Self::valid_masternodes_for_rotated_quorum_map(
                    used_at_h,
                    quorum_modifier,
                    work_block_height,
                );
                // println!("////////////sorted_used_at_h////////////////");
                // println!("{:#?}", sorted_used_at_h.iter().map(|n| n.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>());
                let sorted_unused_at_h = Self::valid_masternodes_for_rotated_quorum_map(
                    unused_at_h,
                    quorum_modifier,
                    work_block_height,
                );
                // println!("////////////sorted_unused_at_h////////////////");
                // println!("{:#?}", sorted_unused_at_h.iter().map(|n| n.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>());
                let mut sorted_combined_mns_list = sorted_unused_at_h;
                sorted_combined_mns_list.extend(sorted_used_at_h);
                // println!("////////////sorted_combined_mns_list////////////////");
                // println!("{:#?}", sorted_combined_mns_list.iter().map(|n| n.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>());
                snapshot.apply_skip_strategy(sorted_combined_mns_list, quorum_count, quarter_size)
            },
            Err(err) => {
                info!("MISSING: snapshot for block at height: {}", work_block_height);
                vec![]
            }
        }
    }

    // fn log_masternodes(vec: &Vec<models::MasternodeEntry>, prefix: String) {
    //     info!("{}", prefix);
    //     vec.iter().for_each(|m| info!("{:?}", m.provider_registration_transaction_hash.reversed()));
    // }

    // Determine quorum members at new index
    pub fn new_quorum_quarter_members(
        &self,
        params: LLMQParams,
        quorum_base_block_height: u32,
        previous_quarters: [&Vec<Vec<models::MasternodeEntry>>; 3],
        cached_lists: &BTreeMap<UInt256, models::MasternodeList>,
        unknown_lists: &mut Vec<UInt256>,
        skip_removed_masternodes: bool,
    ) -> Vec<Vec<models::MasternodeEntry>> {
        let quorum_count = params.signing_active_quorum_count as usize;
        let mut quarter_quorum_members = vec![Vec::<models::MasternodeEntry>::new(); quorum_count];
        let quorum_size = params.size as usize;
        let quarter_size = quorum_size / 4;
        let work_block_height = quorum_base_block_height - 8;
        match self.provider.lookup_block_hash_by_height(work_block_height) {
            Err(err) => panic!("missing block for height: {}: error: {}", work_block_height, err),
            Ok(work_block_hash) => {
                if let Ok(masternode_list) = self.provider.find_masternode_list(work_block_hash, cached_lists, unknown_lists) {
                    //java::generate_masternode_list_from_map(&masternode_list.masternodes);
                    // println!("•••• new_quorum_quarter_members: {:?}: (skip_removed: {}) {}: {}", params.r#type, skip_removed_masternodes, work_block_height, work_block_hash.reversed());
                    // println!("{:#?}", masternode_list);
                    // println!("••••");
                    if masternode_list.masternodes.len() < quarter_size {
                        println!("models list at {}: {} has less masternodes ({}) then required for quarter size: ({})", work_block_height, work_block_hash, masternode_list.masternodes.len(), quarter_size);
                        quarter_quorum_members
                    } else {
                        let mut used_at_h_masternodes = Vec::<models::MasternodeEntry>::new();
                        let mut unused_at_h_masternodes = Vec::<models::MasternodeEntry>::new();
                        let mut used_at_h_indexed_masternodes = vec![Vec::<models::MasternodeEntry>::new(); quorum_count];
                        for i in 0..quorum_count {
                            // for quarters h - c, h -2c, h -3c
                            for quarter in &previous_quarters {
                                if let Some(quarter_nodes) = quarter.get(i) {
                                    //Self::log_masternodes(quarter_nodes, format!("••••• PREV QUARTER {} ••••••• ", i));
                                    for node in quarter_nodes {
                                        let hash = node.provider_registration_transaction_hash;
                                        // let skip = skip_removed_masternodes && !masternode_list.has_masternode(node.provider_registration_transaction_hash);
                                        if (!skip_removed_masternodes || masternode_list.has_masternode(hash)) &&
                                            masternode_list.has_valid_masternode(hash) {
                                            // node.is_valid {
                                            if !used_at_h_masternodes.iter().any(|m| m.provider_registration_transaction_hash == hash) {
                                                used_at_h_masternodes.push(node.clone());
                                            }
                                            if !used_at_h_indexed_masternodes[i].iter().any(|m| m.provider_registration_transaction_hash == hash) {
                                                used_at_h_indexed_masternodes[i].push(node.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        masternode_list.masternodes.values().for_each(|mn| {
                            if mn.is_valid && !used_at_h_masternodes.iter()
                                .any(|node| mn.provider_registration_transaction_hash == node.provider_registration_transaction_hash) {
                                unused_at_h_masternodes.push(mn.clone());
                            }
                        });
                        //Self::log_masternodes(&used_at_h_masternodes, format!("••••• USED AT H {} ••••••• ", work_block_height));
                        //Self::log_masternodes(&unused_at_h_masternodes, format!("••••• UNUSED AT H {} •••••••", work_block_height));
                        let quorum_modifier = models::LLMQEntry::build_llmq_quorum_hash(params.r#type, work_block_hash);
                        let sorted_used_mns_list = Self::valid_masternodes_for_rotated_quorum_map(used_at_h_masternodes, quorum_modifier, work_block_height);
                        let sorted_unused_mns_list = Self::valid_masternodes_for_rotated_quorum_map(unused_at_h_masternodes, quorum_modifier, work_block_height);
                        //Self::log_masternodes(&sorted_unused_mns_list, format!("••••• SORTED UNUSED AT H {} ••••••• ", work_block_height));
                        //Self::log_masternodes(&sorted_used_mns_list, format!("••••• SORTED USED AT H {} ••••••• ", work_block_height));
                        let mut sorted_combined_mns_list = sorted_unused_mns_list;
                        sorted_combined_mns_list.extend(sorted_used_mns_list);
                        // println!("••••• SORTED COMBINED AT H {} •••••••", work_block_height);
                        // println!("{:#?}", sorted_combined_mns_list.iter().map(|m|m.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>());

                        let mut skip_list = Vec::<i32>::new();
                        let mut first_skipped_index = 0i32;
                        let mut idx = 0i32;
                        for i in 0..quorum_count {
                            let masternodes_used_at_h_indexed_at_i = used_at_h_indexed_masternodes.get_mut(i).unwrap();
                            let used_mns_count = masternodes_used_at_h_indexed_at_i.len();
                            let sorted_combined_mns_list_len = sorted_combined_mns_list.len();
                            let mut updated = false;
                            let initial_loop_idx = idx;
                            while quarter_quorum_members[i].len() < quarter_size && used_mns_count + quarter_quorum_members[i].len() < sorted_combined_mns_list_len {
                                let mn = sorted_combined_mns_list.get(idx as usize).unwrap();
                                // TODO: replace masternodes with smart pointers to avoid cloning
                                if masternodes_used_at_h_indexed_at_i.iter().any(|node| mn.provider_registration_transaction_hash == node.provider_registration_transaction_hash) {
                                    let skip_index = idx - first_skipped_index;
                                    if first_skipped_index == 0 {
                                        first_skipped_index = idx;
                                    }
                                    skip_list.push(idx);
                                } else {
                                    masternodes_used_at_h_indexed_at_i.push(mn.clone());
                                    quarter_quorum_members[i].push(mn.clone());
                                    updated = true;
                                }
                                idx += 1;
                                if idx == sorted_combined_mns_list_len as i32 {
                                    idx = 0;
                                }
                                if idx == initial_loop_idx {
                                    if !updated {
                                        println!("there are not enough MNs {}: {} then required for quarter size: ({})", work_block_height, work_block_hash, quarter_size);
                                        return vec![Vec::<models::MasternodeEntry>::new(); quorum_count];
                                    }
                                    updated = false;
                                }
                            }
                        }
                        // println!("••••• QUARTER MEMBERS •••••••");
                        // quarter_quorum_members.iter().enumerate().for_each(|(index, members)| {
                        //     Self::log_masternodes(&members, format!("••••• INDEX {} ••••••• ", index));
                        // });
                        // println!("•••••");
                        quarter_quorum_members
                    }
                } else {
                    println!("missing models list for height: {}: {}", work_block_height, work_block_hash);
                    quarter_quorum_members
                }
            }
        }
    }

    fn add_quorum_members_from_quarter(
        quorum_members: &mut Vec<Vec<models::MasternodeEntry>>,
        quarter: &[Vec<models::MasternodeEntry>],
        index: usize,
    ) {
        if let Some(indexed_quarter) = quarter.get(index) {
            quorum_members.resize_with(index + 1, Vec::new);
            quorum_members[index].extend(indexed_quarter.iter().cloned());
        }
    }

    fn rotate_members(
        &self,
        cycle_quorum_base_block_height: u32,
        llmq_params: LLMQParams,
        cached_lists: &BTreeMap<UInt256, models::MasternodeList>,
        cached_snapshots: &BTreeMap<UInt256, models::LLMQSnapshot>,
        unknown_lists: &mut Vec<UInt256>,
        skip_removed_masternodes: bool,
    ) -> Vec<Vec<models::MasternodeEntry>> {
        let num_quorums = llmq_params.signing_active_quorum_count as usize;
        let cycle_length = llmq_params.dkg_params.interval;
        // println!("/////////////////////// rotate_members {}: {} /////////", cycle_quorum_base_block_height, cycle_length);
        let quorum_base_block_height = cycle_quorum_base_block_height - cycle_length;
        let prev_q_h_m_c = self.quorum_quarter_members_by_snapshot(&llmq_params, quorum_base_block_height, cached_lists, cached_snapshots, unknown_lists);
        // println!("/////////////////////// prev_q_h_m_c : {} /////////", quorum_base_block_height);
        // println!("{:#?}", prev_q_h_m_c.iter().map(|p| p.iter().map(|n| n.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>()).collect::<Vec<_>>());
        let quorum_base_block_height = cycle_quorum_base_block_height - 2 * cycle_length;
        let prev_q_h_m_2c = self.quorum_quarter_members_by_snapshot(&llmq_params, quorum_base_block_height, cached_lists, cached_snapshots, unknown_lists);
        // println!("/////////////////////// prev_q_h_m_2c : {} /////////", quorum_base_block_height);
        // println!("{:#?}", prev_q_h_m_2c.iter().map(|p| p.iter().map(|n| n.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>()).collect::<Vec<_>>());
        let quorum_base_block_height = cycle_quorum_base_block_height - 3 * cycle_length;
        let prev_q_h_m_3c = self.quorum_quarter_members_by_snapshot(&llmq_params, quorum_base_block_height, cached_lists, cached_snapshots, unknown_lists);
        // println!("/////////////////////// prev_q_h_m_3c : {} /////////", quorum_base_block_height);
        // println!("{:#?}", prev_q_h_m_3c.iter().map(|p| p.iter().map(|n| n.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>()).collect::<Vec<_>>());

        let mut rotated_members =
            Vec::<Vec<models::MasternodeEntry>>::with_capacity(num_quorums);
        let new_quarter_members = self.new_quorum_quarter_members(
            llmq_params,
            cycle_quorum_base_block_height,
            [
                &prev_q_h_m_c,
                &prev_q_h_m_2c,
                &prev_q_h_m_3c,
            ],
            cached_lists,
            unknown_lists,
            skip_removed_masternodes,
        );
        // println!("/////////////////////// new_quarter_members : {} /////////", cycle_quorum_base_block_height);
        // println!("{:#?}", new_quarter_members.iter().map(|p| p.iter().map(|n| n.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>()).collect::<Vec<_>>());

        (0..num_quorums).for_each(|i| {
            Self::add_quorum_members_from_quarter(&mut rotated_members, &prev_q_h_m_3c, i);
            Self::add_quorum_members_from_quarter(&mut rotated_members, &prev_q_h_m_2c, i);
            Self::add_quorum_members_from_quarter(&mut rotated_members, &prev_q_h_m_c, i);
            Self::add_quorum_members_from_quarter(&mut rotated_members, &new_quarter_members, i);
        });
        rotated_members
    }

    /// Determine masternodes which is responsible for signing at this quorum index
    #[allow(clippy::too_many_arguments)]
    pub fn get_rotated_masternodes_for_quorum(
        &self,
        llmq_type: LLMQType,
        block_hash: UInt256,
        block_height: u32,
        cached_llmq_members: &mut BTreeMap<LLMQType, BTreeMap<UInt256, Vec<models::MasternodeEntry>>>,
        cached_llmq_indexed_members: &mut BTreeMap<LLMQType, BTreeMap<models::LLMQIndexedHash, Vec<models::MasternodeEntry>>>,
        cached_mn_lists: &BTreeMap<UInt256, models::MasternodeList>,
        cached_llmq_snapshots: &BTreeMap<UInt256, models::LLMQSnapshot>,
        cached_needed_masternode_lists: &mut Vec<UInt256>,
        skip_removed_masternodes: bool,
    ) -> Vec<models::MasternodeEntry> {
        let map_by_type_opt = cached_llmq_members.get_mut(&llmq_type);
        if map_by_type_opt.is_some() {
            if let Some(members) = map_by_type_opt.as_ref().unwrap().get(&block_hash) {
                return members.clone();
            }
        } else {
            cached_llmq_members.insert(llmq_type, BTreeMap::new());
        }
        let map_by_type = cached_llmq_members.get_mut(&llmq_type).unwrap();
        let llmq_params = llmq_type.params();
        let quorum_index = block_height % llmq_params.dkg_params.interval;
        let cycle_base_height = block_height - quorum_index;
        // println!("/////////////////////get_rotated_masternodes_for_quorum {} {} {} {}", block_height, llmq_params.dkg_params.interval, quorum_index, cycle_base_height);
        match self.provider.lookup_block_hash_by_height(cycle_base_height) {
            Err(err) => panic!("missing hash for block at height: {}: error: {}", cycle_base_height, err),
            Ok(cycle_base_hash) => {
                let map_by_type_indexed_opt = cached_llmq_indexed_members.get_mut(&llmq_type);
                if map_by_type_indexed_opt.is_some() {
                    if let Some(members) = map_by_type_indexed_opt
                        .as_ref()
                        .unwrap()
                        .get(&(cycle_base_hash, quorum_index).into())
                    {
                        map_by_type.insert(block_hash, members.clone());
                        return members.clone();
                    }
                } else {
                    cached_llmq_indexed_members.insert(llmq_type, BTreeMap::new());
                }
                let rotated_members = self.rotate_members(
                    cycle_base_height,
                    llmq_params,
                    cached_mn_lists,
                    cached_llmq_snapshots,
                    cached_needed_masternode_lists,
                    skip_removed_masternodes,
                );
                let map_indexed_quorum_members_of_type =
                    cached_llmq_indexed_members.get_mut(&llmq_type).unwrap();
                rotated_members.iter().enumerate().for_each(|(i, members)| {
                    map_indexed_quorum_members_of_type.insert((cycle_base_hash, i).into(), members.clone());
                });
                if let Some(members) = rotated_members.get(quorum_index as usize) {
                    map_by_type.insert(block_hash, members.clone());
                    return members.clone();
                }
                vec![]
            }
        }
    }

    pub fn should_process_quorum(&self, llmq_type: LLMQType, is_dip_0024: bool, is_rotated_quorums_presented: bool) -> bool {
        // TODO: what we really wants here for platform quorum type?
        //is_dip_0024 && llmq_type == LLMQType::Llmqtype60_75
        if self.provider.chain_type().isd_llmq_type() == llmq_type {
            is_dip_0024 && is_rotated_quorums_presented
        } else if is_dip_0024 { /*skip old quorums here for now*/
            false
        } else {
            self.provider.chain_type().should_process_llmq_of_type(llmq_type)
        }
    }

    ///////////////////////////////////////////////////////////////////////////////////////////
    /// FFI-callbacks
    ///////////////////////////////////////////////////////////////////////////////////////////


    pub fn read_list_diff_from_message<'a>(
        &self,
        message: &'a [u8],
        offset: &mut usize,
        protocol_version: u32
    ) -> Result<models::MNListDiff, byte::Error> {
        models::MNListDiff::new(protocol_version, message, offset, |block_hash| self.provider.lookup_block_height_by_hash(block_hash))
    }
    pub fn process_mnlist_diff_internal(&self, message: &[u8], is_from_snapshot: bool, protocol_version: u32, cache: &mut MasternodeProcessorCache) -> Result<MNListDiffResult, ProcessingError> {
        match self.read_list_diff_from_message(message, &mut 0, protocol_version) {
            Ok(list_diff) => {
                if !is_from_snapshot {
                    ok_or_return_processing_error!(self.provider.should_process_diff_with_range(list_diff.base_block_hash, list_diff.block_hash));
                }
                Ok(self.get_list_diff_result_internal_with_base_lookup(list_diff, true, false, false, cache))
            },
            Err(err) => Err(ProcessingError::from(err))
        }
    }
    // There are
    pub fn process_mnlist_diff(&self, message: &[u8], is_from_snapshot: bool, protocol_version: u32, cache: &mut MasternodeProcessorCache) -> Result<types::MNListDiffResult, ProcessingError> {
        match self.read_list_diff_from_message(message, &mut 0, protocol_version) {
            Ok(list_diff) => {
                if !is_from_snapshot {
                    ok_or_return_processing_error!(self.provider.should_process_diff_with_range(list_diff.base_block_hash, list_diff.block_hash));
                }
                Ok(self.get_list_diff_result_with_base_lookup(list_diff, true, false, false, cache))
            },
            Err(err) => Err(ProcessingError::from(err))
        }
    }

    pub fn process_qr_info_internal(&self, message: &[u8], is_from_snapshot: bool, protocol_version: u32, is_rotated_quorums_presented: bool, cache: &mut MasternodeProcessorCache) -> Result<processing::QRInfoResult, ProcessingError> {
        let mut process_list_diff = |list_diff, should_process_quorums|
            self.get_list_diff_result_internal_with_base_lookup(list_diff, should_process_quorums, true, is_rotated_quorums_presented, cache);
        message.read_with::<models::QRInfo>(&mut 0, (&*self.provider, is_from_snapshot, protocol_version, is_rotated_quorums_presented))
            .map_err(ProcessingError::from)
            .map(|qr_info| processing::QRInfoResult {
                result_at_h_4c: qr_info.diff_h_4c.map(|list_diff| process_list_diff(list_diff, false)),
                result_at_h_3c: process_list_diff(qr_info.diff_h_3c, false),
                result_at_h_2c: process_list_diff(qr_info.diff_h_2c, false),
                result_at_h_c: process_list_diff(qr_info.diff_h_c, false),
                result_at_h: process_list_diff(qr_info.diff_h, true),
                result_at_tip: process_list_diff(qr_info.diff_tip, false),
                snapshot_at_h_c: qr_info.snapshot_h_c,
                snapshot_at_h_2c: qr_info.snapshot_h_2c,
                snapshot_at_h_3c: qr_info.snapshot_h_3c,
                snapshot_at_h_4c: qr_info.snapshot_h_4c,
                extra_share: qr_info.extra_share,
                last_quorum_per_index: qr_info.last_quorum_per_index,
                quorum_snapshot_list: qr_info.quorum_snapshot_list,
                mn_list_diff_list: qr_info.mn_list_diff_list
                    .into_iter()
                    .map(|list_diff| process_list_diff(list_diff, false))
                    .collect()
            })
    }

    pub fn process_qr_info(&self, message: &[u8], is_from_snapshot: bool, protocol_version: u32,
        is_rotated_quorums_presented: bool, cache: &mut MasternodeProcessorCache) -> Result<types::QRInfoResult, ProcessingError> {
        // self.process_qr_info_internal(message, is_from_snapshot, protocol_version, is_rotated_quorums_presented, cache)
        let offset = &mut 0;


        let mut process_list_diff = |list_diff: models::MNListDiff, should_process_quorums: bool| {
            self.get_list_diff_result_with_base_lookup(list_diff, should_process_quorums, true, is_rotated_quorums_presented, cache)
        };

        let read_list_diff =
            |offset: &mut usize| self.read_list_diff_from_message(message, offset, protocol_version);
        let read_snapshot = |offset: &mut usize| models::LLMQSnapshot::from_bytes(message, offset);
        let read_var_int = |offset: &mut usize| encode::VarInt::from_bytes(message, offset);
        let mut get_list_diff_result =
            |list_diff: models::MNListDiff, verify_quorums: bool| boxed(process_list_diff(list_diff, verify_quorums));

        let snapshot_at_h_c = ok_or_return_processing_error!(read_snapshot(offset));
        let snapshot_at_h_2c = ok_or_return_processing_error!(read_snapshot(offset));
        let snapshot_at_h_3c = ok_or_return_processing_error!(read_snapshot(offset));
        let diff_tip = ok_or_return_processing_error!(read_list_diff(offset));
        if !is_from_snapshot {
            ok_or_return_processing_error!(self.provider.should_process_diff_with_range(diff_tip.base_block_hash, diff_tip.block_hash));
        }
        let diff_h = ok_or_return_processing_error!(read_list_diff(offset));
        let diff_h_c = ok_or_return_processing_error!(read_list_diff(offset));
        let diff_h_2c = ok_or_return_processing_error!(read_list_diff(offset));
        let diff_h_3c = ok_or_return_processing_error!(read_list_diff(offset));
        let extra_share = message.read_with::<bool>(offset, ()).unwrap_or(false);
        let (snapshot_at_h_4c, diff_h_4c) = if extra_share {
            let snapshot_at_h_4c = ok_or_return_processing_error!(read_snapshot(offset));
            let diff_h_4c = ok_or_return_processing_error!(read_list_diff(offset));
            (Some(snapshot_at_h_4c), Some(diff_h_4c))
        } else {
            (None, None)
        };
        self.provider.save_snapshot(diff_h_c.block_hash, snapshot_at_h_c.clone());
        self.provider.save_snapshot(diff_h_2c.block_hash, snapshot_at_h_2c.clone());
        self.provider.save_snapshot(diff_h_3c.block_hash, snapshot_at_h_3c.clone());
        if extra_share {
            self.provider.save_snapshot(diff_h_4c.as_ref().unwrap().block_hash, snapshot_at_h_4c.clone().unwrap());
        }

        let last_quorum_per_index_count = ok_or_return_processing_error!(read_var_int(offset)).0 as usize;
        let mut last_quorum_per_index_vec: Vec<*mut types::LLMQEntry> =
            Vec::with_capacity(last_quorum_per_index_count);
        for _i in 0..last_quorum_per_index_count {
            let quorum = ok_or_return_processing_error!(models::LLMQEntry::from_bytes(message, offset));
            last_quorum_per_index_vec.push(boxed(quorum.encode()));
        }
        let quorum_snapshot_list_count = ok_or_return_processing_error!(read_var_int(offset)).0 as usize;
        let mut quorum_snapshot_list_vec: Vec<*mut types::LLMQSnapshot> =
            Vec::with_capacity(quorum_snapshot_list_count);
        let mut snapshots: Vec<models::LLMQSnapshot> = Vec::with_capacity(quorum_snapshot_list_count);
        for _i in 0..quorum_snapshot_list_count {
            let snapshot = ok_or_return_processing_error!(read_snapshot(offset));
            snapshots.push(snapshot);
        }
        let mn_list_diff_list_count = ok_or_return_processing_error!(read_var_int(offset)).0 as usize;
        let mut mn_list_diff_list_vec: Vec<*mut types::MNListDiffResult> =
            Vec::with_capacity(mn_list_diff_list_count);
        assert_eq!(quorum_snapshot_list_count, mn_list_diff_list_count, "'quorum_snapshot_list_count' must be equal 'mn_list_diff_list_count'");
        for i in 0..mn_list_diff_list_count {
            let list_diff = ok_or_return_processing_error!(read_list_diff(offset));
            let block_hash = list_diff.block_hash;
            mn_list_diff_list_vec.push(get_list_diff_result(list_diff, false));
            let snapshot = snapshots.get(i).unwrap();
            quorum_snapshot_list_vec.push(boxed(snapshot.encode()));
            self.provider.save_snapshot(block_hash, snapshot.clone());
        }

        let result_at_h_4c = if extra_share {
            get_list_diff_result(diff_h_4c.unwrap(), false)
        } else {
            std::ptr::null_mut()
        };
        let result_at_h_3c = get_list_diff_result(diff_h_3c, false);
        let result_at_h_2c = get_list_diff_result(diff_h_2c, false);
        let result_at_h_c = get_list_diff_result(diff_h_c, false);
        let result_at_h = get_list_diff_result(diff_h, true);
        let result_at_tip = get_list_diff_result(diff_tip, false);
        let result = types::QRInfoResult {
            error_status: ProcessingError::None,
            result_at_tip,
            result_at_h,
            result_at_h_c,
            result_at_h_2c,
            result_at_h_3c,
            result_at_h_4c,
            snapshot_at_h_c: boxed(snapshot_at_h_c.encode()),
            snapshot_at_h_2c: boxed(snapshot_at_h_2c.encode()),
            snapshot_at_h_3c: boxed(snapshot_at_h_3c.encode()),
            snapshot_at_h_4c: if extra_share {
                boxed(snapshot_at_h_4c.unwrap().encode())
            } else {
                std::ptr::null_mut()
            },
            extra_share,
            last_quorum_per_index: rs_ffi_interfaces::boxed_vec(last_quorum_per_index_vec),
            last_quorum_per_index_count,
            quorum_snapshot_list: rs_ffi_interfaces::boxed_vec(quorum_snapshot_list_vec),
            quorum_snapshot_list_count,
            mn_list_diff_list: rs_ffi_interfaces::boxed_vec(mn_list_diff_list_vec),
            mn_list_diff_list_count,
        };
        Ok(result)
    }
}
