// Copyright 2019-2024 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{
    rpc::eth::types::EthAddress,
    shim::crypto::{Signature, SignatureType},
};
use anyhow::{ensure, Context};
use derive_builder::Builder;
use num::BigInt;
use num_bigint::Sign;
use num_traits::cast::ToPrimitive;

pub const HOMESTEAD_SIG_LEN: usize = 66;
pub const HOMESTEAD_SIG_PREFIX: u8 = 0x01;

#[derive(PartialEq, Debug, Clone, Default, Builder)]
#[builder(setter(into))]
pub struct EthLegacyHomesteadTxArgs {
    pub nonce: u64,
    pub gas_price: BigInt,
    pub gas_limit: u64,
    pub to: Option<EthAddress>,
    pub value: BigInt,
    pub input: Vec<u8>,
    #[builder(setter(skip))]
    pub v: BigInt,
    #[builder(setter(skip))]
    pub r: BigInt,
    #[builder(setter(skip))]
    pub s: BigInt,
}
impl EthLegacyHomesteadTxArgs {
    pub(crate) fn with_signature(mut self, signature: &Signature) -> anyhow::Result<Self> {
        ensure!(
            signature.signature_type() == SignatureType::Delegated,
            "Signature is not delegated type"
        );

        ensure!(
            signature.bytes().len() == HOMESTEAD_SIG_LEN,
            "Invalid signature length for Homestead transaction"
        );

        ensure!(
            signature.bytes().first().expect("infallible") == &HOMESTEAD_SIG_PREFIX,
            "Invalid signature prefix for Homestead transaction"
        );

        // ignore the first byte of the signature as it's only used for legacy transaction identification
        let r = BigInt::from_bytes_be(
            Sign::Plus,
            signature.bytes().get(1..33).expect("infallible"),
        );
        let s = BigInt::from_bytes_be(
            Sign::Plus,
            signature.bytes().get(33..65).expect("infallible"),
        );
        let v = BigInt::from_bytes_be(Sign::Plus, signature.bytes().get(65..).expect("infallible"));

        let v_int = v.to_i32().context("Failed to convert v to i32")?;
        ensure!(
            v_int == 27 || v_int == 28,
            "Homestead transaction v value is invalid"
        );

        self.r = r;
        self.s = s;
        self.v = v;

        Ok(self)
    }
}
