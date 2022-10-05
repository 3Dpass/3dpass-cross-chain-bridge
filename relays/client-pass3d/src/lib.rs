// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of Parity Bridges Common.

// Parity Bridges Common is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Bridges Common is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Bridges Common.  If not, see <http://www.gnu.org/licenses/>.

//! Types used to connect to the Pass3d-Substrate chain.

use bp_messages::MessageNonce;
use codec::{Compact, Decode, Encode};
use frame_support::weights::Weight;
use relay_substrate_client::{
	BalanceOf, Chain, ChainBase, ChainWithBalances, ChainWithGrandpa, ChainWithMessages,
	Error as SubstrateError, IndexOf, SignParam, TransactionSignScheme,
	UnsignedTransaction,
};
use sp_core::{storage::StorageKey, Pair};
use sp_runtime::{generic::SignedPayload, traits::IdentifyAccount};
use std::time::Duration;

/// Pass3d header id.
pub type HeaderId = relay_utils::HeaderId<pass3d_runtime::Hash, pass3d_runtime::BlockNumber>;

/// Pass3d chain definition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pass3d;

impl ChainBase for Pass3d {
	type BlockNumber = pass3d_runtime::BlockNumber;
	type Hash = pass3d_runtime::Hash;
	type Hasher = pass3d_runtime::Hashing;
	type Header = pass3d_runtime::Header;

	type AccountId = pass3d_runtime::AccountId;
	type Balance = pass3d_runtime::Balance;
	type Index = pass3d_runtime::Index;
	type Signature = pass3d_runtime::Signature;

	fn max_extrinsic_size() -> u32 {
		bp_pass3d::Pass3d::max_extrinsic_size()
	}

	fn max_extrinsic_weight() -> Weight {
		bp_pass3d::Pass3d::max_extrinsic_weight()
	}
}

impl Chain for Pass3d {
	const NAME: &'static str = "Pass3d";
	// Pass3d token has no value, but we associate it with DOT token
	const TOKEN_ID: Option<&'static str> = Some("polkadot");
	const BEST_FINALIZED_HEADER_ID_METHOD: &'static str =
		bp_pass3d::BEST_FINALIZED_PASS3D_HEADER_METHOD;
	const AVERAGE_BLOCK_INTERVAL: Duration = Duration::from_secs(5);
	const STORAGE_PROOF_OVERHEAD: u32 = bp_pass3d::EXTRA_STORAGE_PROOF_SIZE;

	type SignedBlock = pass3d_runtime::SignedBlock;
	type Call = pass3d_runtime::Call;
}
//
// impl RelayChain for Pass3d {
// 	const PARAS_PALLET_NAME: &'static str = bp_pass3d::PARAS_PALLET_NAME;
// 	const PARACHAINS_FINALITY_PALLET_NAME: &'static str =
// 		bp_pass3d::WITH_PASS3D_BRIDGE_PARAS_PALLET_NAME;
// }

impl ChainWithGrandpa for Pass3d {
	const WITH_CHAIN_GRANDPA_PALLET_NAME: &'static str = bp_pass3d::WITH_PASS3D_GRANDPA_PALLET_NAME;
}

impl ChainWithMessages for Pass3d {
	const WITH_CHAIN_MESSAGES_PALLET_NAME: &'static str =
		bp_pass3d::WITH_PASS3D_MESSAGES_PALLET_NAME;
	const TO_CHAIN_MESSAGE_DETAILS_METHOD: &'static str =
		bp_pass3d::TO_PASS3D_MESSAGE_DETAILS_METHOD;
	const FROM_CHAIN_MESSAGE_DETAILS_METHOD: &'static str =
		bp_pass3d::FROM_PASS3D_MESSAGE_DETAILS_METHOD;
	const PAY_INBOUND_DISPATCH_FEE_WEIGHT_AT_CHAIN: Weight =
		bp_pass3d::PAY_INBOUND_DISPATCH_FEE_WEIGHT;
	const MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX: MessageNonce =
		bp_pass3d::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	const MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX: MessageNonce =
		bp_pass3d::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	type WeightToFee = bp_pass3d::WeightToFee;
	type WeightInfo = ();
}

impl ChainWithBalances for Pass3d {
	fn account_info_storage_key(account_id: &Self::AccountId) -> StorageKey {
		use frame_support::storage::generator::StorageMap;
		StorageKey(frame_system::Account::<pass3d_runtime::Runtime>::storage_map_final_key(
			account_id,
		))
	}
}

impl TransactionSignScheme for Pass3d {
	type Chain = Pass3d;
	type AccountKeyPair = sp_core::sr25519::Pair;
	type SignedTransaction = pass3d_runtime::UncheckedExtrinsic;

	fn sign_transaction(
		param: SignParam<Self>,
		unsigned: UnsignedTransaction<Self::Chain>,
	) -> Result<Self::SignedTransaction, SubstrateError> {
		let raw_payload = SignedPayload::from_raw(
			unsigned.call.clone(),
			(
				frame_system::CheckNonZeroSender::<pass3d_runtime::Runtime>::new(),
				frame_system::CheckSpecVersion::<pass3d_runtime::Runtime>::new(),
				frame_system::CheckTxVersion::<pass3d_runtime::Runtime>::new(),
				frame_system::CheckGenesis::<pass3d_runtime::Runtime>::new(),
				frame_system::CheckEra::<pass3d_runtime::Runtime>::from(unsigned.era.frame_era()),
				frame_system::CheckNonce::<pass3d_runtime::Runtime>::from(unsigned.nonce),
				frame_system::CheckWeight::<pass3d_runtime::Runtime>::new(),
				pallet_transaction_payment::ChargeTransactionPayment::<pass3d_runtime::Runtime>::from(unsigned.tip),
			),
			(
				(),
				param.spec_version,
				param.transaction_version,
				param.genesis_hash,
				unsigned.era.signed_payload(param.genesis_hash),
				(),
				(),
				(),
			),
		);
		let signature = raw_payload.using_encoded(|payload| param.signer.sign(payload));
		let signer: sp_runtime::MultiSigner = param.signer.public().into();
		let (call, extra, _) = raw_payload.deconstruct();

		Ok(pass3d_runtime::UncheckedExtrinsic::new_signed(
			call.into_decoded()?,
			signer.into_account().into(),
			signature.into(),
			extra,
		))
	}

	fn is_signed(tx: &Self::SignedTransaction) -> bool {
		tx.signature.is_some()
	}

	fn is_signed_by(signer: &Self::AccountKeyPair, tx: &Self::SignedTransaction) -> bool {
		tx.signature
			.as_ref()
			.map(|(address, _, _)| *address == pass3d_runtime::Address::Id(signer.public().into()))
			.unwrap_or(false)
	}

	fn parse_transaction(tx: Self::SignedTransaction) -> Option<UnsignedTransaction<Self::Chain>> {
		let extra = &tx.signature.as_ref()?.2;
		Some(
			UnsignedTransaction::new(
				tx.function.into(),
				Compact::<IndexOf<Self::Chain>>::decode(&mut &extra.5.encode()[..]).ok()?.into(),
			)
			.tip(
				Compact::<BalanceOf<Self::Chain>>::decode(&mut &extra.7.encode()[..])
					.ok()?
					.into(),
			),
		)
	}
}

/// Pass3d signing params.
pub type SigningParams = sp_core::sr25519::Pair;

/// Pass3d header type used in headers sync.
pub type SyncHeader = relay_substrate_client::SyncHeader<pass3d_runtime::Header>;

#[cfg(test)]
mod tests {
	use super::*;
	use relay_substrate_client::TransactionEra;

	#[test]
	fn parse_transaction_works() {
		let unsigned = UnsignedTransaction {
			call: pass3d_runtime::Call::System(pass3d_runtime::SystemCall::remark {
				remark: b"Hello world!".to_vec(),
			})
			.into(),
			nonce: 777,
			tip: 888,
			era: TransactionEra::immortal(),
		};
		let signed_transaction = Pass3d::sign_transaction(
			SignParam {
				spec_version: 42,
				transaction_version: 50000,
				genesis_hash: [42u8; 32].into(),
				signer: sp_core::sr25519::Pair::from_seed_slice(&[1u8; 32]).unwrap(),
			},
			unsigned.clone(),
		)
		.unwrap();
		let parsed_transaction = Pass3d::parse_transaction(signed_transaction).unwrap();
		assert_eq!(parsed_transaction, unsigned);
	}
}
