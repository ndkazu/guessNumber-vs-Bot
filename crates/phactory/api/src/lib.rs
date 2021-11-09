#![no_std]
extern crate alloc;

pub mod crypto;
pub mod ecall_args;

mod proto_generated;

pub mod prpc {
    pub use crate::proto_generated::*;
    use alloc::vec::Vec;
    use phala_types::messaging::{MessageOrigin, SignedMessage};
    pub use prpc::{client, server, Message};
    pub type EgressMessages = Vec<(MessageOrigin, Vec<SignedMessage>)>;

    pub const SIG_LEN: usize = 64;
}

#[cfg(feature = "pruntime-client")]
pub mod pruntime_client;

pub mod actions {
    pub const ACTION_GET_INFO: u8 = 2;

    pub const BIN_ACTION_START: u8 = 128;
    pub const BIN_ACTION_SYNC_PARA_HEADER: u8 = BIN_ACTION_START;
    pub const BIN_ACTION_DISPATCH_BLOCK: u8 = BIN_ACTION_START + 1;
    pub const BIN_ACTION_SYNC_HEADER: u8 = BIN_ACTION_START + 2;
    pub const BIN_ACTION_SYNC_COMBINED_HEADERS: u8 = BIN_ACTION_START + 3;
}

pub mod blocks {
    use alloc::vec::Vec;
    use core::convert::TryFrom;
    use parity_scale_codec::{Decode, Encode, FullCodec};
    pub use sp_finality_grandpa::{AuthorityList, SetId};

    use sp_core::U256;
    use sp_runtime::{generic::Header, traits::Hash as HashT};
    pub use phala_trie_storage::ser::StorageChanges;

    pub type StorageProof = Vec<Vec<u8>>;
    pub type StorageState = Vec<(Vec<u8>, Vec<u8>)>;

    /// The GRNADPA authority set with the id
    #[derive(Encode, Decode, Clone, PartialEq, Debug)]
    pub struct AuthoritySet {
        pub list: AuthorityList,
        pub id: SetId,
    }

    /// AuthoritySet change with the storage proof (including both the authority set and the id)
    #[derive(Encode, Decode, Clone, PartialEq, Debug)]
    pub struct AuthoritySetChange {
        pub authority_set: AuthoritySet,
        pub authority_proof: StorageProof,
    }

    /// The genesis block initialization info.
    ///
    /// The genesis block is the first block to start GRNADPA light validation tracking. It could
    /// be block 0 or a later block on the relay chain. The authority set represents the validator
    /// infomation at the selected block.
    #[derive(Encode, Decode, Clone, PartialEq, Debug)]
    pub struct GenesisBlockInfo {
        pub block_header: chain::Header,
        pub authority_set: AuthoritySet,
        pub proof: StorageProof,
    }

    pub type RuntimeHasher = <chain::Runtime as frame_system::Config>::Hashing;
    pub type HeaderToSync = GenericHeaderToSync<chain::BlockNumber, RuntimeHasher>;
    pub type BlockHeaderWithChanges =
        GenericBlockHeaderWithChanges<chain::BlockNumber, RuntimeHasher>;
    pub type Headers = Vec<Header<chain::BlockNumber, RuntimeHasher>>;
    pub type HeadersToSync = Vec<HeaderToSync>;

    pub type RawStorageKey = Vec<u8>;

    #[derive(Debug, Encode, Decode, Clone)]
    pub struct StorageKV<T: FullCodec + Clone>(pub RawStorageKey, pub T);

    impl<T: FullCodec + Clone> StorageKV<T> {
        pub fn key(&self) -> &RawStorageKey {
            &self.0
        }
        pub fn value(&self) -> &T {
            &self.1
        }
    }

    #[derive(Encode, Decode, Debug, Clone)]
    pub struct GenericHeaderToSync<BlockNumber, Hash>
    where
        BlockNumber: Copy + Into<U256> + TryFrom<U256> + Clone,
        Hash: HashT,
    {
        pub header: Header<BlockNumber, Hash>,
        pub justification: Option<Vec<u8>>,
    }

    #[derive(Encode, Decode, Clone, Debug)]
    pub struct GenericBlockHeaderWithChanges<BlockNumber, Hash>
    where
        BlockNumber: Copy + Into<U256> + TryFrom<U256> + FullCodec + Clone,
        Hash: HashT,
    {
        pub block_header: Header<BlockNumber, Hash>,
        pub storage_changes: StorageChanges,
    }

    #[derive(Encode, Decode, Clone, Debug)]
    pub struct SyncHeaderReq {
        pub headers: Vec<HeaderToSync>,
        pub authority_set_change: Option<AuthoritySetChange>,
    }

    // TODO.kevin: import it from some other crate
    #[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
    pub struct ParaId(u32);

    impl ParaId {
        pub fn new(n: u32) -> ParaId {
            ParaId(n)
        }
    }

    #[derive(Encode, Decode, Clone, Debug)]
    pub struct SyncParachainHeaderReq {
        pub headers: Headers,
        pub proof: StorageProof,
    }

    #[derive(Encode, Decode, Clone, Debug)]
    pub struct SyncCombinedHeadersReq {
        pub relaychain_headers: Vec<HeaderToSync>,
        pub authority_set_change: Option<AuthoritySetChange>,
        pub parachain_headers: Headers,
        pub proof: StorageProof,
    }

    #[derive(Encode, Decode, Clone, Debug)]
    pub struct DispatchBlockReq {
        pub blocks: Vec<BlockHeaderWithChanges>,
    }

    #[cfg(feature = "serde")]
    pub mod compat {
        use alloc::string::String;
        use alloc::vec::Vec;
        use parity_scale_codec::Encode;
        use serde::Serialize;

        #[derive(Serialize, Debug)]
        pub struct SyncHeaderReq {
            pub headers_b64: Vec<String>,
            pub authority_set_change_b64: Option<String>,
        }

        impl From<super::SyncHeaderReq> for SyncHeaderReq {
            fn from(v: super::SyncHeaderReq) -> Self {
                let headers_b64: Vec<_> = v
                    .headers
                    .into_iter()
                    .map(|x| base64::encode(x.encode()))
                    .collect();
                let authority_set_change_b64 =
                    v.authority_set_change.map(|x| base64::encode(x.encode()));
                Self {
                    headers_b64,
                    authority_set_change_b64,
                }
            }
        }

        #[derive(Serialize, Debug)]
        pub struct DispatchBlockReq {
            pub blocks_b64: Vec<String>,
        }

        impl From<super::DispatchBlockReq> for DispatchBlockReq {
            fn from(v: super::DispatchBlockReq) -> Self {
                let blocks_b64: Vec<_> = v
                    .blocks
                    .into_iter()
                    .map(|x| base64::encode(x.encode()))
                    .collect();
                Self { blocks_b64 }
            }
        }

        #[derive(Serialize, Debug)]
        pub struct ContractInput<T> {
            pub input: T,
        }

        impl<T> ContractInput<T> {
            pub fn new(input: T) -> Self {
                Self { input }
            }
        }
    }
}

pub mod storage_sync {
    use super::blocks::{
        AuthoritySetChange, BlockHeaderWithChanges, HeaderToSync, RuntimeHasher, StorageProof,
    };

    use alloc::collections::VecDeque;
    use alloc::string::String;
    use alloc::vec::Vec;
    use chain::Hash;
    use derive_more::Display;
    use parity_scale_codec::Encode;

    type Storage = phala_trie_storage::TrieStorage<RuntimeHasher>;

    pub type Result<T> = core::result::Result<T, Error>;

    #[derive(Display)]
    pub enum Error {
        /// No header or block data in the request
        EmptyRequest,
        /// No Justification found in the last header
        MissingJustification,
        /// Header validation failed
        #[display(fmt = "HeaderValidateFailed({})", .0)]
        HeaderValidateFailed(String),
        /// Storage proof failed
        #[display(fmt = "StorageProofFailed({})", .0)]
        StorageProofFailed(String),
        /// Relay chain header not synced before syncing parachain header
        RelaychainHeaderNotSynced,
        /// Some header parent hash mismatches it's parent header
        HeaderHashMismatch,
        /// The end block number mismatch the expecting next block number
        BlockNumberMismatch,
        /// No state root to validate the storage changes
        NoStateRoot,
        /// Invalid storage changes that cause the state root mismatch
        #[display(
            fmt = "StateRootMismatch block={:?} expected={:?} actual={:?}",
            block,
            expected,
            actual
        )]
        StateRootMismatch {
            block: chain::BlockNumber,
            expected: chain::Hash,
            actual: chain::Hash,
        },
        /// Solo/Para mode mismatch
        ChainModeMismatch,
    }

    pub trait BlockValidator {
        fn submit_finalized_headers(
            &mut self,
            bridge_id: u64,
            header: chain::Header,
            ancestry_proof: Vec<chain::Header>,
            grandpa_proof: Vec<u8>,
            auhtority_set_change: Option<AuthoritySetChange>,
        ) -> Result<()>;

        fn validate_storage_proof(
            &self,
            state_root: Hash,
            proof: StorageProof,
            items: &[(&[u8], &[u8])],
        ) -> Result<()>;
    }

    pub trait StorageSynchronizer {
        /// Return the next block numbers to sync.
        fn counters(&self) -> Counters;

        /// Given chain headers in sequence, validate it and output the state_roots
        fn sync_header(
            &mut self,
            headers: Vec<HeaderToSync>,
            authority_set_change: Option<AuthoritySetChange>,
        ) -> Result<chain::BlockNumber>;

        /// Given the parachain headers in sequence, validate it and cached the state_roots for block validation
        fn sync_parachain_header(
            &mut self,
            headers: Vec<chain::Header>,
            proof: StorageProof,
            storage_key: &[u8],
        ) -> Result<chain::BlockNumber>;

        /// Feed in a block of storage changes
        fn feed_block(
            &mut self,
            block: &BlockHeaderWithChanges,
            storage: &mut Storage,
        ) -> Result<()>;
    }

    pub struct BlockSyncState<Validator> {
        validator: Validator,
        main_bridge: u64,
        /// The next block number of header to be sync.
        header_number_next: chain::BlockNumber,
        /// The next block number of storage to be sync.
        /// Note: in parachain mode the `block_number_next` is for the parachain and the
        /// header_number_next is for the relaychain.
        block_number_next: chain::BlockNumber,
    }

    impl<Validator> BlockSyncState<Validator>
    where
        Validator: BlockValidator,
    {
        pub fn new(
            validator: Validator,
            main_bridge: u64,
            header_number_next: chain::BlockNumber,
            block_number_next: chain::BlockNumber,
        ) -> Self {
            Self {
                validator,
                main_bridge,
                header_number_next,
                block_number_next,
            }
        }

        /// Given chain headers in sequence, validate it and output the state_roots
        pub fn sync_header(
            &mut self,
            headers: Vec<HeaderToSync>,
            authority_set_change: Option<AuthoritySetChange>,
            state_roots: &mut VecDeque<Hash>,
        ) -> Result<chain::BlockNumber> {
            let first_header = headers.first().ok_or(Error::EmptyRequest)?;
            if first_header.header.number != self.header_number_next {
                return Err(Error::BlockNumberMismatch);
            }

            // Light validation when possible
            let last_header = headers.last().ok_or(Error::EmptyRequest)?;

            {
                // 1. the last header must has justification
                let justification = last_header
                    .justification
                    .as_ref()
                    .ok_or(Error::MissingJustification)?
                    .clone();
                let last_header = last_header.header.clone();
                // 2. check header sequence
                for (i, header) in headers.iter().enumerate() {
                    if i > 0 && headers[i - 1].header.hash() != header.header.parent_hash {
                        return Err(Error::HeaderHashMismatch);
                    }
                }
                // 3. generate accenstor proof
                let mut accenstor_proof: Vec<_> = headers[0..headers.len() - 1]
                    .iter()
                    .map(|h| h.header.clone())
                    .collect();
                accenstor_proof.reverse(); // from high to low
                                           // 4. submit to light client
                let bridge_id = self.main_bridge;
                self.validator.submit_finalized_headers(
                    bridge_id,
                    last_header,
                    accenstor_proof,
                    justification,
                    authority_set_change,
                )?;
            }

            // Save the block hashes for future dispatch
            for header in headers.iter() {
                state_roots.push_back(header.header.state_root);
            }

            self.header_number_next = last_header.header.number + 1;

            Ok(last_header.header.number)
        }

        /// Feed a block and apply changes to storage if it's valid.
        pub fn feed_block(
            &mut self,
            block: &BlockHeaderWithChanges,
            state_roots: &mut VecDeque<Hash>,
            storage: &mut Storage,
        ) -> Result<()> {
            if block.block_header.number != self.block_number_next {
                return Err(Error::BlockNumberMismatch);
            }

            let expected_root = state_roots.get(0).ok_or(Error::NoStateRoot)?;

            let changes = &block.storage_changes;

            let (state_root, transaction) = storage.calc_root_if_changes(
                &changes.main_storage_changes,
                &changes.child_storage_changes,
            );

            if expected_root != &state_root {
                return Err(Error::StateRootMismatch {
                    block: block.block_header.number,
                    expected: *expected_root,
                    actual: state_root,
                });
            }

            storage.apply_changes(state_root, transaction);

            self.block_number_next += 1;
            state_roots.pop_front();
            Ok(())
        }
    }

    #[derive(Default, Debug)]
    pub struct Counters {
        pub next_header_number: chain::BlockNumber,
        pub next_para_header_number: chain::BlockNumber,
        pub next_block_number: chain::BlockNumber,
    }

    pub struct SolochainSynchronizer<Validator> {
        sync_state: BlockSyncState<Validator>,
        state_roots: VecDeque<Hash>,
    }

    impl<Validator: BlockValidator> SolochainSynchronizer<Validator> {
        pub fn new(validator: Validator, main_bridge: u64) -> Self {
            Self {
                sync_state: BlockSyncState::new(validator, main_bridge, 1, 1),
                state_roots: Default::default(),
            }
        }
    }

    impl<Validator: BlockValidator> StorageSynchronizer for SolochainSynchronizer<Validator> {
        fn counters(&self) -> Counters {
            Counters {
                next_block_number: self.sync_state.block_number_next,
                next_header_number: self.sync_state.header_number_next,
                next_para_header_number: 0,
            }
        }

        fn sync_header(
            &mut self,
            headers: Vec<HeaderToSync>,
            authority_set_change: Option<AuthoritySetChange>,
        ) -> Result<chain::BlockNumber> {
            self.sync_state
                .sync_header(headers, authority_set_change, &mut self.state_roots)
        }

        fn feed_block(
            &mut self,
            block: &BlockHeaderWithChanges,
            storage: &mut Storage,
        ) -> Result<()> {
            self.sync_state
                .feed_block(block, &mut self.state_roots, storage)
        }

        fn sync_parachain_header(
            &mut self,
            _headers: Vec<chain::Header>,
            _proof: StorageProof,
            _storage_key: &[u8],
        ) -> Result<chain::BlockNumber> {
            Err(Error::ChainModeMismatch)
        }
    }

    pub struct ParachainSynchronizer<Validator> {
        sync_state: BlockSyncState<Validator>,
        last_relaychain_state_root: Option<Hash>,
        para_header_number_next: chain::BlockNumber,
        para_state_roots: VecDeque<Hash>,
    }

    impl<Validator: BlockValidator> ParachainSynchronizer<Validator> {
        pub fn new(
            validator: Validator,
            main_bridge: u64,
            headernum_next: chain::BlockNumber,
        ) -> Self {
            Self {
                sync_state: BlockSyncState::new(validator, main_bridge, headernum_next, 1),
                last_relaychain_state_root: None,
                para_header_number_next: 1,
                para_state_roots: Default::default(),
            }
        }
    }

    impl<Validator: BlockValidator> StorageSynchronizer for ParachainSynchronizer<Validator> {
        fn counters(&self) -> Counters {
            Counters {
                next_block_number: self.sync_state.block_number_next,
                next_header_number: self.sync_state.header_number_next,
                next_para_header_number: self.para_header_number_next,
            }
        }

        /// Given the relaychain headers in sequence, validate it and cached the last state_root for parachain header validation
        fn sync_header(
            &mut self,
            headers: Vec<HeaderToSync>,
            authority_set_change: Option<AuthoritySetChange>,
        ) -> Result<chain::BlockNumber> {
            let mut state_roots = Default::default();
            let last_header =
                self.sync_state
                    .sync_header(headers, authority_set_change, &mut state_roots)?;
            self.last_relaychain_state_root = state_roots.pop_back();
            Ok(last_header)
        }

        /// Given the parachain headers in sequence, validate it and cached the state_roots for block validation
        fn sync_parachain_header(
            &mut self,
            headers: Vec<chain::Header>,
            proof: StorageProof,
            storage_key: &[u8],
        ) -> Result<chain::BlockNumber> {
            let first_hdr = headers.first().ok_or(Error::EmptyRequest)?;
            if self.para_header_number_next != first_hdr.number {
                return Err(Error::BlockNumberMismatch);
            }

            let last_hdr = headers.last().ok_or(Error::EmptyRequest)?;

            let state_root = self
                .last_relaychain_state_root
                .as_ref()
                .cloned()
                .ok_or(Error::RelaychainHeaderNotSynced)?;

            // 1. validate storage proof
            self.sync_state.validator.validate_storage_proof(
                state_root,
                proof,
                &[(storage_key, last_hdr.encode().encode().as_slice())],
            )?;

            // 2. check header sequence
            for (i, header) in headers.iter().enumerate() {
                if i > 0 && headers[i - 1].hash() != header.parent_hash {
                    return Err(Error::HeaderHashMismatch);
                }
            }

            // All checks passed, enqueue the state roots for storage validation.
            for hdr in headers.iter() {
                self.para_state_roots.push_back(hdr.state_root);
            }

            self.last_relaychain_state_root = None;
            self.para_header_number_next = last_hdr.number + 1;

            Ok(last_hdr.number)
        }

        /// Feed in a block of storage changes
        fn feed_block(
            &mut self,
            block: &BlockHeaderWithChanges,
            storage: &mut Storage,
        ) -> Result<()> {
            self.sync_state
                .feed_block(block, &mut self.para_state_roots, storage)
        }
    }

    #[cfg(test)]
    mod tests {
        // TODO.kevin: Write some tests
    }
}
