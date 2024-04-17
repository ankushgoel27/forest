// Copyright 2019-2024 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

//! Types that are shared _between_ APIs.
//!
//! If a type here is used by only one API, it should be relocated.

use crate::blocks::TipsetKey;
use crate::libp2p::Multihash;
use crate::lotus_json::{lotus_json_with_self, HasLotusJson, LotusJson};
use crate::shim::{
    address::Address,
    clock::ChainEpoch,
    deal::DealID,
    econ::TokenAmount,
    executor::Receipt,
    fvm_shared_latest::MethodNum,
    message::Message,
    sector::{RegisteredSealProof, SectorNumber},
};
use cid::Cid;
use fil_actor_interface::market::AllocationID;
use fil_actor_interface::miner::MinerInfo;
use fil_actor_interface::{
    market::{DealProposal, DealState},
    miner::MinerPower,
    power::Claim,
};
use fil_actor_miner_state::v12::{BeneficiaryTerm, PendingBeneficiaryChange};
use fil_actors_shared::fvm_ipld_bitfield::BitField;
use fvm_ipld_encoding::{BytesDe, RawBytes};
use libipld_core::ipld::Ipld;
use libp2p::PeerId;
use nonempty::NonEmpty;
use num_bigint::BigInt;
use schemars::JsonSchema;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
#[cfg(test)]
use serde_json::Value;
use std::str::FromStr;

// Chain API

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct MessageSendSpec {
    max_fee: LotusJson<TokenAmount>,
}

lotus_json_with_self!(MessageSendSpec);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct ApiDealState {
    pub sector_start_epoch: ChainEpoch,
    pub last_updated_epoch: ChainEpoch,
    pub slash_epoch: ChainEpoch,
    #[serde(skip)]
    pub verified_claim: AllocationID,
}

lotus_json_with_self!(ApiDealState);

impl From<DealState> for ApiDealState {
    fn from(s: DealState) -> Self {
        let DealState {
            sector_start_epoch,
            last_updated_epoch,
            slash_epoch,
            verified_claim,
        } = s;
        Self {
            sector_start_epoch,
            last_updated_epoch,
            slash_epoch,
            verified_claim,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct ApiDealProposal {
    #[serde(rename = "PieceCID", with = "crate::lotus_json")]
    pub piece_cid: Cid,
    pub piece_size: u64,
    pub verified_deal: bool,
    #[serde(with = "crate::lotus_json")]
    pub client: Address,
    #[serde(with = "crate::lotus_json")]
    pub provider: Address,
    pub label: String,
    pub start_epoch: ChainEpoch,
    pub end_epoch: ChainEpoch,
    #[serde(with = "crate::lotus_json")]
    pub storage_price_per_epoch: TokenAmount,
    #[serde(with = "crate::lotus_json")]
    pub provider_collateral: TokenAmount,
    #[serde(with = "crate::lotus_json")]
    pub client_collateral: TokenAmount,
}

lotus_json_with_self!(ApiDealProposal);

impl From<DealProposal> for ApiDealProposal {
    fn from(p: DealProposal) -> Self {
        let DealProposal {
            piece_cid,
            piece_size,
            verified_deal,
            client,
            provider,
            label,
            start_epoch,
            end_epoch,
            storage_price_per_epoch,
            provider_collateral,
            client_collateral,
        } = p;
        Self {
            piece_cid,
            piece_size: piece_size.0,
            verified_deal,
            client: client.into(),
            provider: provider.into(),
            label,
            start_epoch,
            end_epoch,
            storage_price_per_epoch: storage_price_per_epoch.into(),
            provider_collateral: provider_collateral.into(),
            client_collateral: client_collateral.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct ApiMarketDeal {
    #[serde(with = "crate::lotus_json")]
    pub proposal: ApiDealProposal,
    #[serde(with = "crate::lotus_json")]
    pub state: ApiDealState,
}

lotus_json_with_self!(ApiMarketDeal);

impl From<MarketDeal> for ApiMarketDeal {
    fn from(d: MarketDeal) -> Self {
        Self {
            proposal: d.proposal.into(),
            state: d.state.into(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MarketDeal {
    pub proposal: DealProposal,
    pub state: DealState,
}

// TODO(elmattic): https://github.com/ChainSafe/fil-actor-states/issues/255
//                 Remove this impl
impl Clone for MarketDeal {
    fn clone(&self) -> Self {
        Self {
            proposal: DealProposal {
                piece_cid: self.proposal.piece_cid,
                piece_size: self.proposal.piece_size,
                verified_deal: self.proposal.verified_deal,
                client: self.proposal.client,
                provider: self.proposal.provider,
                label: self.proposal.label.clone(),
                start_epoch: self.proposal.start_epoch,
                end_epoch: self.proposal.end_epoch,
                storage_price_per_epoch: self.proposal.storage_price_per_epoch.clone(),
                provider_collateral: self.proposal.provider_collateral.clone(),
                client_collateral: self.proposal.client_collateral.clone(),
            },
            state: DealState { ..self.state },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MessageLookup {
    #[serde(with = "crate::lotus_json")]
    pub receipt: Receipt,
    #[serde(rename = "TipSet", with = "crate::lotus_json")]
    pub tipset: TipsetKey,
    pub height: i64,
    #[serde(with = "crate::lotus_json")]
    pub message: Cid,
    #[serde(with = "crate::lotus_json")]
    pub return_dec: Ipld,
}

lotus_json_with_self!(MessageLookup);

#[derive(Serialize, Deserialize)]
pub struct PeerID {
    pub multihash: Multihash,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct ApiTipsetKey(pub Option<TipsetKey>);

impl From<TipsetKey> for ApiTipsetKey {
    fn from(value: TipsetKey) -> Self {
        Self(Some(value))
    }
}

impl From<&TipsetKey> for ApiTipsetKey {
    fn from(value: &TipsetKey) -> Self {
        value.clone().into()
    }
}

impl HasLotusJson for ApiTipsetKey {
    type LotusJson = LotusJson<Vec<Cid>>;

    #[cfg(test)]
    fn snapshots() -> Vec<(serde_json::Value, Self)> {
        vec![]
    }

    fn into_lotus_json(self) -> Self::LotusJson {
        LotusJson(
            self.0
                .map(|ts| ts.into_cids().into_iter().collect::<Vec<Cid>>())
                .unwrap_or_default(),
        )
    }

    fn from_lotus_json(LotusJson(lotus_json): Self::LotusJson) -> Self {
        Self(NonEmpty::from_vec(lotus_json).map(From::from))
    }
}

impl std::fmt::Display for ApiTipsetKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(tsk) = &self.0 {
            write!(f, "{tsk}")
        } else {
            write!(f, "")
        }
    }
}

const EMPTY_ADDRESS_VALUE: &str = "<empty>";

/// This wrapper is needed because of a bug in Lotus.
/// See: <https://github.com/filecoin-project/lotus/issues/11461>.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AddressOrEmpty(pub Option<Address>);

impl Serialize for AddressOrEmpty {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let address_bytes = match self.0 {
            Some(addr) => addr.to_string(),
            None => EMPTY_ADDRESS_VALUE.to_string(),
        };

        s.collect_str(&address_bytes)
    }
}

impl<'de> Deserialize<'de> for AddressOrEmpty {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let address_str = String::deserialize(deserializer)?;
        if address_str.eq(EMPTY_ADDRESS_VALUE) {
            return Ok(Self(None));
        }

        Address::from_str(&address_str)
            .map_err(de::Error::custom)
            .map(|addr| Self(Some(addr)))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MinerInfoLotusJson {
    #[serde(with = "crate::lotus_json")]
    pub owner: Address,
    #[serde(with = "crate::lotus_json")]
    pub worker: Address,
    pub new_worker: AddressOrEmpty,
    #[serde(with = "crate::lotus_json")]
    pub control_addresses: Vec<Address>, // Must all be ID addresses.
    pub worker_change_epoch: ChainEpoch,
    #[serde(with = "crate::lotus_json")]
    pub peer_id: Option<String>,
    #[serde(with = "crate::lotus_json")]
    pub multiaddrs: Vec<Vec<u8>>,
    pub window_po_st_proof_type: fvm_shared2::sector::RegisteredPoStProof,
    pub sector_size: fvm_shared2::sector::SectorSize,
    pub window_po_st_partition_sectors: u64,
    pub consensus_fault_elapsed: ChainEpoch,
    #[serde(with = "crate::lotus_json")]
    pub pending_owner_address: Option<Address>,
    #[serde(with = "crate::lotus_json")]
    pub beneficiary: Address,
    #[serde(with = "crate::lotus_json")]
    pub beneficiary_term: BeneficiaryTerm,
    #[serde(with = "crate::lotus_json")]
    pub pending_beneficiary_term: Option<PendingBeneficiaryChange>,
}

impl HasLotusJson for MinerInfo {
    type LotusJson = MinerInfoLotusJson;
    #[cfg(test)]
    fn snapshots() -> Vec<(serde_json::Value, Self)> {
        vec![]
    }
    fn into_lotus_json(self) -> Self::LotusJson {
        MinerInfoLotusJson {
            owner: self.owner.into(),
            worker: self.worker.into(),
            new_worker: AddressOrEmpty(self.new_worker.map(|addr| addr.into())),
            control_addresses: self
                .control_addresses
                .into_iter()
                .map(|a| a.into())
                .collect(),
            worker_change_epoch: self.worker_change_epoch,
            peer_id: PeerId::try_from(self.peer_id).map(|id| id.to_base58()).ok(),
            multiaddrs: self.multiaddrs.into_iter().map(|addr| addr.0).collect(),
            window_po_st_proof_type: self.window_post_proof_type,
            sector_size: self.sector_size,
            window_po_st_partition_sectors: self.window_post_partition_sectors,
            consensus_fault_elapsed: self.consensus_fault_elapsed,
            // NOTE: In Lotus this field is never set for any of the versions, so we have to ignore
            // it too.
            // See: <https://github.com/filecoin-project/lotus/blob/b6a77dfafcf0110e95840fca15a775ed663836d8/chain/actors/builtin/miner/v12.go#L370>.
            pending_owner_address: None,
            beneficiary: self.beneficiary.into(),
            beneficiary_term: self.beneficiary_term,
            pending_beneficiary_term: self.pending_beneficiary_term,
        }
    }
    fn from_lotus_json(lotus_json: Self::LotusJson) -> Self {
        MinerInfo {
            owner: lotus_json.owner.into(),
            worker: lotus_json.worker.into(),
            new_worker: lotus_json.new_worker.0.map(|addr| addr.into()),
            control_addresses: lotus_json
                .control_addresses
                .into_iter()
                .map(|a| a.into())
                .collect(),
            worker_change_epoch: lotus_json.worker_change_epoch,
            peer_id: lotus_json.peer_id.map_or_else(Vec::new, |s| s.into_bytes()),
            multiaddrs: lotus_json.multiaddrs.into_iter().map(BytesDe).collect(),
            window_post_proof_type: lotus_json.window_po_st_proof_type,
            sector_size: lotus_json.sector_size,
            window_post_partition_sectors: lotus_json.window_po_st_partition_sectors,
            consensus_fault_elapsed: lotus_json.consensus_fault_elapsed,
            // Ignore this field as it is never set on Lotus side.
            pending_owner_address: None,
            beneficiary: lotus_json.beneficiary.into(),
            beneficiary_term: lotus_json.beneficiary_term,
            pending_beneficiary_term: lotus_json.pending_beneficiary_term,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BeneficiaryTermLotusJson {
    /// The total amount the current beneficiary can withdraw. Monotonic, but reset when beneficiary changes.
    #[serde(with = "crate::lotus_json")]
    pub quota: TokenAmount,
    /// The amount of quota the current beneficiary has already withdrawn
    #[serde(with = "crate::lotus_json")]
    pub used_quota: TokenAmount,
    /// The epoch at which the beneficiary's rights expire and revert to the owner
    pub expiration: ChainEpoch,
}

impl HasLotusJson for BeneficiaryTerm {
    type LotusJson = BeneficiaryTermLotusJson;

    #[cfg(test)]
    fn snapshots() -> Vec<(Value, Self)> {
        vec![]
    }

    fn into_lotus_json(self) -> Self::LotusJson {
        BeneficiaryTermLotusJson {
            used_quota: self.used_quota.into(),
            quota: self.quota.into(),
            expiration: self.expiration,
        }
    }

    fn from_lotus_json(lotus_json: Self::LotusJson) -> Self {
        Self {
            used_quota: lotus_json.used_quota.into(),
            quota: lotus_json.quota.into(),
            expiration: lotus_json.expiration,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PendingBeneficiaryChangeLotusJson {
    #[serde(with = "crate::lotus_json")]
    pub new_beneficiary: Address,
    #[serde(with = "crate::lotus_json")]
    pub new_quota: TokenAmount,
    pub new_expiration: ChainEpoch,
    pub approved_by_beneficiary: bool,
    pub approved_by_nominee: bool,
}

impl HasLotusJson for PendingBeneficiaryChange {
    type LotusJson = PendingBeneficiaryChangeLotusJson;

    #[cfg(test)]
    fn snapshots() -> Vec<(Value, Self)> {
        vec![]
    }

    fn into_lotus_json(self) -> Self::LotusJson {
        PendingBeneficiaryChangeLotusJson {
            new_beneficiary: self.new_beneficiary.into(),
            new_quota: self.new_quota.into(),
            new_expiration: self.new_expiration,
            approved_by_beneficiary: self.approved_by_beneficiary,
            approved_by_nominee: self.approved_by_nominee,
        }
    }

    fn from_lotus_json(lotus_json: Self::LotusJson) -> Self {
        Self {
            new_beneficiary: lotus_json.new_beneficiary.into(),
            new_quota: lotus_json.new_quota.into(),
            new_expiration: lotus_json.new_expiration,
            approved_by_beneficiary: lotus_json.approved_by_beneficiary,
            approved_by_nominee: lotus_json.approved_by_nominee,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MinerPowerLotusJson {
    miner_power: LotusJson<Claim>,
    total_power: LotusJson<Claim>,
    has_min_power: bool,
}

impl HasLotusJson for MinerPower {
    type LotusJson = MinerPowerLotusJson;
    #[cfg(test)]
    fn snapshots() -> Vec<(serde_json::Value, Self)> {
        vec![]
    }
    fn into_lotus_json(self) -> Self::LotusJson {
        MinerPowerLotusJson {
            miner_power: LotusJson(self.miner_power),
            total_power: LotusJson(self.total_power),
            has_min_power: self.has_min_power,
        }
    }
    fn from_lotus_json(lotus_json: Self::LotusJson) -> Self {
        MinerPower {
            miner_power: lotus_json.miner_power.into_inner(),
            total_power: lotus_json.total_power.into_inner(),
            has_min_power: lotus_json.has_min_power,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ApiActorState {
    #[serde(with = "crate::lotus_json")]
    balance: TokenAmount,
    #[serde(with = "crate::lotus_json")]
    code: Cid,
    #[serde(with = "crate::lotus_json")]
    state: ApiState,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
struct ApiState {
    #[serde(rename = "BuiltinActors")]
    #[serde(with = "crate::lotus_json")]
    state: Ipld,
}

lotus_json_with_self!(ApiState);
lotus_json_with_self!(ApiActorState);

impl ApiActorState {
    pub fn new(balance: TokenAmount, code: Cid, state: Ipld) -> Self {
        Self {
            balance,
            code,
            state: ApiState { state },
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct SectorOnChainInfo {
    pub sector_number: SectorNumber,

    /// The seal proof type implies the PoSt proofs
    pub seal_proof: RegisteredSealProof,

    #[serde(with = "crate::lotus_json")]
    #[serde(rename = "SealedCID")]
    /// `CommR`
    pub sealed_cid: Cid,

    #[serde(rename = "DealIDs")]
    #[serde(with = "crate::lotus_json")]
    pub deal_ids: Vec<DealID>,

    /// Epoch during which the sector proof was accepted
    pub activation: ChainEpoch,

    /// Epoch during which the sector expires
    pub expiration: ChainEpoch,

    #[serde(with = "crate::lotus_json")]
    /// Integral of active deals over sector lifetime
    pub deal_weight: BigInt,

    #[serde(with = "crate::lotus_json")]
    /// Integral of active verified deals over sector lifetime
    pub verified_deal_weight: BigInt,

    #[serde(with = "crate::lotus_json")]
    /// Pledge collected to commit this sector
    pub initial_pledge: TokenAmount,

    #[serde(with = "crate::lotus_json")]
    /// Expected one day projection of reward for sector computed at activation
    /// time
    pub expected_day_reward: TokenAmount,

    #[serde(with = "crate::lotus_json")]
    /// Expected twenty day projection of reward for sector computed at
    /// activation time
    pub expected_storage_pledge: TokenAmount,

    pub replaced_sector_age: ChainEpoch,

    #[serde(with = "crate::lotus_json")]
    pub replaced_day_reward: TokenAmount,

    #[serde(with = "crate::lotus_json")]
    #[serde(rename = "SectorKeyCID")]
    pub sector_key_cid: Option<Cid>,

    #[serde(rename = "SimpleQAPower")]
    pub simple_qa_power: bool,
}

impl From<fil_actor_interface::miner::SectorOnChainInfo> for SectorOnChainInfo {
    fn from(other: fil_actor_interface::miner::SectorOnChainInfo) -> Self {
        SectorOnChainInfo {
            sector_number: other.sector_number,
            seal_proof: other.seal_proof.into(),
            sealed_cid: other.sealed_cid,
            deal_ids: other.deal_ids,
            activation: other.activation,
            expiration: other.expiration,
            deal_weight: other.deal_weight,
            verified_deal_weight: other.verified_deal_weight,
            initial_pledge: other.initial_pledge.into(),
            expected_day_reward: other.expected_day_reward.into(),
            expected_storage_pledge: other.expected_storage_pledge.into(),
            replaced_sector_age: other.replaced_sector_age,
            // `replaced_day_reward` has to be zero and Lemmih cannot figure out
            // why. Lotus casts all `SectorOnChainInfo` structs to the miner-v9
            // version which clears some fields (like `simple_qa_power`) but it
            // shouldn't clear `replaced_day_reward`. Oh well, maybe one day
            // Lemmih will figure it out.
            replaced_day_reward: TokenAmount::default(),
            sector_key_cid: other.sector_key_cid,
            simple_qa_power: other.simple_qa_power,
        }
    }
}

lotus_json_with_self!(SectorOnChainInfo);

#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ApiDeadline {
    #[serde(with = "crate::lotus_json")]
    pub post_submissions: BitField,
    #[serde(with = "crate::lotus_json")]
    pub disputable_proof_count: u64,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct MinerSectors {
    live: u64,
    active: u64,
    faulty: u64,
}

impl MinerSectors {
    pub fn new(live: u64, active: u64, faulty: u64) -> Self {
        Self {
            live,
            active,
            faulty,
        }
    }
}

lotus_json_with_self!(MinerSectors);

#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct MinerPartitions {
    #[serde(with = "crate::lotus_json")]
    all_sectors: BitField,
    #[serde(with = "crate::lotus_json")]
    faulty_sectors: BitField,
    #[serde(with = "crate::lotus_json")]
    recovering_sectors: BitField,
    #[serde(with = "crate::lotus_json")]
    live_sectors: BitField,
    #[serde(with = "crate::lotus_json")]
    active_sectors: BitField,
}

impl MinerPartitions {
    pub fn new(
        all_sectors: &BitField,
        faulty_sectors: &BitField,
        recovering_sectors: &BitField,
        live_sectors: BitField,
        active_sectors: BitField,
    ) -> Self {
        Self {
            all_sectors: all_sectors.clone(),
            faulty_sectors: faulty_sectors.clone(),
            recovering_sectors: recovering_sectors.clone(),
            live_sectors,
            active_sectors,
        }
    }
}

lotus_json_with_self!(MinerPartitions);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MessageFilter {
    #[serde(with = "crate::lotus_json")]
    pub from: Option<Address>,
    #[serde(with = "crate::lotus_json")]
    pub to: Option<Address>,
}

impl MessageFilter {
    pub fn matches(&self, msg: &Message) -> bool {
        if let Some(from) = &self.from {
            if from != &msg.from {
                return false;
            }
        }

        if let Some(to) = &self.to {
            if to != &msg.to {
                return false;
            }
        }

        true
    }

    pub fn is_empty(&self) -> bool {
        self.from.is_none() && self.to.is_none()
    }
}

lotus_json_with_self!(MessageFilter);

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Transaction {
    #[serde(rename = "ID")]
    pub id: i64,
    #[serde(with = "crate::lotus_json")]
    pub to: Address,
    #[serde(with = "crate::lotus_json")]
    pub value: TokenAmount,
    pub method: MethodNum,
    #[serde(with = "crate::lotus_json")]
    pub params: RawBytes,
    #[serde(with = "crate::lotus_json")]
    pub approved: Vec<Address>,
}

lotus_json_with_self!(Transaction);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DealCollateralBounds {
    #[serde(with = "crate::lotus_json")]
    pub min: TokenAmount,
    #[serde(with = "crate::lotus_json")]
    pub max: TokenAmount,
}

lotus_json_with_self!(DealCollateralBounds);

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn test_api_tipset_key(cids: Vec<Cid>) {
        test_api_tipset_key_inner(cids)
    }

    #[test]
    fn test_api_tipset_key_empty() {
        test_api_tipset_key_inner(vec![])
    }

    #[test]
    fn test_api_tipset_key_deserialization_empty_vec() {
        let api_ts_lotus_json: LotusJson<ApiTipsetKey> = serde_json::from_str("[]").unwrap();
        assert!(api_ts_lotus_json.into_inner().0.is_none());
    }

    #[test]
    fn test_api_tipset_key_deserialization_null() {
        let api_ts_lotus_json: LotusJson<ApiTipsetKey> = serde_json::from_str("null").unwrap();
        assert!(api_ts_lotus_json.into_inner().0.is_none());
    }

    fn test_api_tipset_key_inner(cids: Vec<Cid>) {
        let cids_lotus_json = LotusJson(cids.clone());
        let lotus_json_str = serde_json::to_string_pretty(&cids_lotus_json).unwrap();
        let api_ts_lotus_json: LotusJson<ApiTipsetKey> =
            serde_json::from_str(&lotus_json_str).unwrap();
        let api_ts = api_ts_lotus_json.into_inner();
        let cids_from_api_ts = api_ts
            .0
            .map(|ts| ts.into_cids().into_iter().collect::<Vec<Cid>>())
            .unwrap_or_default();
        assert_eq!(cids_from_api_ts, cids);
    }
}
