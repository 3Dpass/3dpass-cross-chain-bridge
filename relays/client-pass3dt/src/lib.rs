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

//! Types used to connect to the Pass3dt-Substrate chain.

use bp_messages::MessageNonce;
use codec::{Compact, Decode, Encode};
use frame_support::weights::Weight;
use relay_substrate_client::{
	BalanceOf, Chain, ChainBase, ChainWithBalances, ChainWithGrandpa, ChainWithMessages,
	Error as SubstrateError, IndexOf, SignParam, TransactionSignScheme, UnsignedTransaction,
};
use sp_core::{storage::StorageKey, Pair};
use sp_runtime::{generic::SignedPayload, traits::IdentifyAccount};
use std::time::Duration;

/// Pass3dt header id.
pub type HeaderId = relay_utils::HeaderId<pass3dt_runtime::Hash, pass3dt_runtime::BlockNumber>;

/// Pass3dt chain definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pass3dt;

impl ChainBase for Pass3dt {
	type BlockNumber = pass3dt_runtime::BlockNumber;
	type Hash = pass3dt_runtime::Hash;
	type Hasher = pass3dt_runtime::Hashing;
	type Header = pass3dt_runtime::Header;

	type AccountId = pass3dt_runtime::AccountId;
	type Balance = pass3dt_runtime::Balance;
	type Index = pass3dt_runtime::Index;
	type Signature = pass3dt_runtime::Signature;

	fn max_extrinsic_size() -> u32 {
		bp_pass3dt::Pass3dt::max_extrinsic_size()
	}

	fn max_extrinsic_weight() -> Weight {
		bp_pass3dt::Pass3dt::max_extrinsic_weight()
	}
}

impl ChainWithGrandpa for Pass3dt {
	const WITH_CHAIN_GRANDPA_PALLET_NAME: &'static str = bp_pass3dt::WITH_PASS3DT_GRANDPA_PALLET_NAME;
}

impl ChainWithMessages for Pass3dt {
	const WITH_CHAIN_MESSAGES_PALLET_NAME: &'static str =
		bp_pass3dt::WITH_PASS3DT_MESSAGES_PALLET_NAME;
	const TO_CHAIN_MESSAGE_DETAILS_METHOD: &'static str =
		bp_pass3dt::TO_PASS3DT_MESSAGE_DETAILS_METHOD;
	const FROM_CHAIN_MESSAGE_DETAILS_METHOD: &'static str =
		bp_pass3dt::FROM_PASS3DT_MESSAGE_DETAILS_METHOD;
	const PAY_INBOUND_DISPATCH_FEE_WEIGHT_AT_CHAIN: Weight =
		bp_pass3dt::PAY_INBOUND_DISPATCH_FEE_WEIGHT;
	const MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX: MessageNonce =
		bp_pass3dt::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	const MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX: MessageNonce =
		bp_pass3dt::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	type WeightToFee = bp_pass3dt::WeightToFee;
	type WeightInfo = ();
}

impl Chain for Pass3dt {
	const NAME: &'static str = "Pass3dt";
	// Pass3d token has no value, but we associate it with KSM token
	const TOKEN_ID: Option<&'static str> = Some("kusama");
	const BEST_FINALIZED_HEADER_ID_METHOD: &'static str =
		bp_pass3dt::BEST_FINALIZED_PASS3DT_HEADER_METHOD;
	const AVERAGE_BLOCK_INTERVAL: Duration = Duration::from_secs(5);
	const STORAGE_PROOF_OVERHEAD: u32 = bp_pass3dt::EXTRA_STORAGE_PROOF_SIZE;

	type SignedBlock = pass3dt_runtime::SignedBlock;
	type Call = pass3dt_runtime::Call;
}

impl ChainWithBalances for Pass3dt {
	fn account_info_storage_key(account_id: &Self::AccountId) -> StorageKey {
		use frame_support::storage::generator::StorageMap;
		StorageKey(frame_system::Account::<pass3dt_runtime::Runtime>::storage_map_final_key(
			account_id,
		))
	}
}

impl TransactionSignScheme for Pass3dt {
	type Chain = Pass3dt;
	type AccountKeyPair = sp_core::sr25519::Pair;
	type SignedTransaction = pass3dt_runtime::UncheckedExtrinsic;

	fn sign_transaction(
		param: SignParam<Self>,
		unsigned: UnsignedTransaction<Self::Chain>,
	) -> Result<Self::SignedTransaction, SubstrateError> {
		let raw_payload = SignedPayload::from_raw(
			unsigned.call.clone(),
			(
				frame_system::CheckNonZeroSender::<pass3dt_runtime::Runtime>::new(),
				frame_system::CheckSpecVersion::<pass3dt_runtime::Runtime>::new(),
				frame_system::CheckTxVersion::<pass3dt_runtime::Runtime>::new(),
				frame_system::CheckGenesis::<pass3dt_runtime::Runtime>::new(),
				frame_system::CheckEra::<pass3dt_runtime::Runtime>::from(unsigned.era.frame_era()),
				frame_system::CheckNonce::<pass3dt_runtime::Runtime>::from(unsigned.nonce),
				frame_system::CheckWeight::<pass3dt_runtime::Runtime>::new(),
				pallet_transaction_payment::ChargeTransactionPayment::<pass3dt_runtime::Runtime>::from(unsigned.tip),
				pass3dt_runtime::BridgeRejectObsoleteHeadersAndMessages,
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
				(),
			),
		);
		let signature = raw_payload.using_encoded(|payload| param.signer.sign(payload));
		let signer: sp_runtime::MultiSigner = param.signer.public().into();
		let (call, extra, _) = raw_payload.deconstruct();

		Ok(pass3dt_runtime::UncheckedExtrinsic::new_signed(
			call.into_decoded()?,
			signer.into_account(),
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
			.map(|(address, _, _)| {
				*address == pass3dt_runtime::Address::from(*signer.public().as_array_ref())
			})
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

/// Pass3dt signing params.
pub type SigningParams = sp_core::sr25519::Pair;

/// Pass3dt header type used in headers sync.
pub type SyncHeader = relay_substrate_client::SyncHeader<pass3dt_runtime::Header>;

#[cfg(test)]
mod tests {
	use super::*;
	use relay_substrate_client::TransactionEra;

	#[test]
	fn parse_transaction_works() {
		let unsigned = UnsignedTransaction {
			call: pass3dt_runtime::Call::System(pass3dt_runtime::SystemCall::remark {
				remark: b"Hello world!".to_vec(),
			})
			.into(),
			nonce: 777,
			tip: 888,
			era: TransactionEra::immortal(),
		};
		let signed_transaction = Pass3dt::sign_transaction(
			SignParam {
				spec_version: 42,
				transaction_version: 50000,
				genesis_hash: [42u8; 64].into(),
				signer: sp_core::sr25519::Pair::from_seed_slice(&[1u8; 32]).unwrap(),
			},
			unsigned.clone(),
		)
		.unwrap();
		let parsed_transaction = Pass3dt::parse_transaction(signed_transaction).unwrap();
		assert_eq!(parsed_transaction, unsigned);
	}
}
