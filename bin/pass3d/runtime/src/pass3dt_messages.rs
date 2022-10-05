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

//! Everything required to serve Pass3dt <-> Pass3d messages.

use crate::{Call, OriginCaller, Runtime};

use bp_messages::{
	source_chain::{SenderOrigin, TargetHeaderChain},
	target_chain::{ProvedMessages, SourceHeaderChain},
	InboundLaneData, LaneId, Message, MessageNonce, Parameter as MessagesParameter,
};
use bp_runtime::{Chain, ChainId, PASS3DT_CHAIN_ID, PASS3D_CHAIN_ID};
use bridge_runtime_common::messages::{
	self, BasicConfirmationTransactionEstimation, MessageBridge, MessageTransaction,
};
use codec::{Decode, Encode};
use frame_support::{
	parameter_types,
	weights::{DispatchClass, Weight},
	RuntimeDebug,
};
use scale_info::TypeInfo;
use sp_runtime::{traits::Saturating, FixedPointNumber, FixedU128};
use sp_std::convert::TryFrom;

/// Initial value of `Pass3dtToPass3dConversionRate` parameter.
pub const INITIAL_PASS3DT_TO_PASS3D_CONVERSION_RATE: FixedU128 =
	FixedU128::from_inner(FixedU128::DIV);
/// Initial value of `Pass3dtFeeMultiplier` parameter.
pub const INITIAL_PASS3DT_FEE_MULTIPLIER: FixedU128 = FixedU128::from_inner(FixedU128::DIV);
/// Weight of 2 XCM instructions is for simple `Trap(42)` program, coming through bridge
/// (it is prepended with `UniversalOrigin` instruction). It is used just for simplest manual
/// tests, confirming that we don't break encoding somewhere between.
pub const BASE_XCM_WEIGHT_TWICE: Weight = 2 * crate::xcm_config::BASE_XCM_WEIGHT;

parameter_types! {
	/// Pass3dt to Pass3d conversion rate. Initially we treat both tokens as equal.
	pub storage Pass3dtToPass3dConversionRate: FixedU128 = INITIAL_PASS3DT_TO_PASS3D_CONVERSION_RATE;
	/// Fee multiplier value at Pass3dt chain.
	pub storage Pass3dtFeeMultiplier: FixedU128 = INITIAL_PASS3DT_FEE_MULTIPLIER;
}

/// Message payload for Pass3d -> Pass3dt messages.
pub type ToPass3dtMessagePayload = messages::source::FromThisChainMessagePayload;

/// Message verifier for Pass3d -> Pass3dt messages.
pub type ToPass3dtMessageVerifier =
	messages::source::FromThisChainMessageVerifier<WithPass3dtMessageBridge>;

/// Message payload for Pass3dt -> Pass3d messages.
pub type FromPass3dtMessagePayload = messages::target::FromBridgedChainMessagePayload<Call>;

/// Call-dispatch based message dispatch for Pass3dt -> Pass3d messages.
pub type FromPass3dtMessageDispatch = messages::target::FromBridgedChainMessageDispatch<
	WithPass3dtMessageBridge,
	xcm_executor::XcmExecutor<crate::xcm_config::XcmConfig>,
	crate::xcm_config::XcmWeigher,
	//
	frame_support::traits::ConstU64<BASE_XCM_WEIGHT_TWICE>,
>;

/// Messages proof for Pass3dt -> Pass3d messages.
pub type FromPass3dtMessagesProof = messages::target::FromBridgedChainMessagesProof<bp_pass3dt::Hash>;

/// Messages delivery proof for Pass3d -> Pass3dt messages.
pub type ToPass3dtMessagesDeliveryProof =
	messages::source::FromBridgedChainMessagesDeliveryProof<bp_pass3dt::Hash>;

/// Maximal outbound payload size of Pass3d -> Pass3dt messages.
pub type ToPass3dtMaximalOutboundPayloadSize =
	messages::source::FromThisChainMaximalOutboundPayloadSize<WithPass3dtMessageBridge>;

/// Pass3dt <-> Pass3d message bridge.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct WithPass3dtMessageBridge;

impl MessageBridge for WithPass3dtMessageBridge {
	const RELAYER_FEE_PERCENT: u32 = 10;
	const THIS_CHAIN_ID: ChainId = PASS3D_CHAIN_ID;
	const BRIDGED_CHAIN_ID: ChainId = PASS3DT_CHAIN_ID;
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str = bp_pass3d::WITH_PASS3D_MESSAGES_PALLET_NAME;

	type ThisChain = Pass3d;
	type BridgedChain = Pass3dt;

	fn bridged_balance_to_this_balance(
		bridged_balance: bp_pass3dt::Balance,
		bridged_to_this_conversion_rate_override: Option<FixedU128>,
	) -> bp_pass3d::Balance {
		let conversion_rate = bridged_to_this_conversion_rate_override
			.unwrap_or_else(Pass3dtToPass3dConversionRate::get);
		bp_pass3d::Balance::try_from(conversion_rate.saturating_mul_int(bridged_balance))
			.unwrap_or(bp_pass3d::Balance::MAX)
	}
}

/// Pass3d chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct Pass3d;

impl messages::ChainWithMessages for Pass3d {
	type Hash = bp_pass3d::Hash;
	type AccountId = bp_pass3d::AccountId;
	type Signer = bp_pass3d::AccountSigner;
	type Signature = bp_pass3d::Signature;
	type Weight = Weight;
	type Balance = bp_pass3d::Balance;
}

impl messages::ThisChainWithMessages for Pass3d {
	type Origin = crate::Origin;
	type Call = crate::Call;
	type ConfirmationTransactionEstimation = BasicConfirmationTransactionEstimation<
		Self::AccountId,
		{ bp_pass3d::MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT },
		{ bp_pass3dt::EXTRA_STORAGE_PROOF_SIZE },
		{ bp_pass3d::TX_EXTRA_BYTES },
	>;

	fn is_message_accepted(send_origin: &Self::Origin, lane: &LaneId) -> bool {
		let here_location =
			xcm::v3::MultiLocation::from(crate::xcm_config::UniversalLocation::get());
		match send_origin.caller {
			OriginCaller::XcmPallet(pallet_xcm::Origin::Xcm(ref location))
				if *location == here_location =>
			{
				log::trace!(target: "runtime::bridge", "Verifying message sent using XCM pallet to Pass3dt");
			},
			_ => {
				// keep in mind that in this case all messages are free (in term of fees)
				// => it's just to keep testing bridge on our test deployments until we'll have a
				// better option
				log::trace!(target: "runtime::bridge", "Verifying message sent using messages pallet to Pass3dt");
			},
		}

		*lane == [0, 0, 0, 0] || *lane == [0, 0, 0, 1]
	}

	fn maximal_pending_messages_at_outbound_lane() -> MessageNonce {
		MessageNonce::MAX
	}

	fn transaction_payment(transaction: MessageTransaction<Weight>) -> bp_pass3d::Balance {
		// `transaction` may represent transaction from the future, when multiplier value will
		// be larger, so let's use slightly increased value
		let multiplier = FixedU128::saturating_from_rational(110, 100)
			.saturating_mul(pallet_transaction_payment::Pallet::<Runtime>::next_fee_multiplier());
		// in our testnets, both per-byte fee and weight-to-fee are 1:1
		messages::transaction_payment(
			bp_pass3d::BlockWeights::get().get(DispatchClass::Normal).base_extrinsic,
			1,
			multiplier,
			|weight| weight as _,
			transaction,
		)
	}
}

/// Pass3dt chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct Pass3dt;

impl messages::ChainWithMessages for Pass3dt {
	type Hash = bp_pass3dt::Hash;
	type AccountId = bp_pass3dt::AccountId;
	type Signer = bp_pass3dt::AccountSigner;
	type Signature = bp_pass3dt::Signature;
	type Weight = Weight;
	type Balance = bp_pass3dt::Balance;
}

impl messages::BridgedChainWithMessages for Pass3dt {
	fn maximal_extrinsic_size() -> u32 {
		bp_pass3dt::Pass3dt::max_extrinsic_size()
	}

	fn verify_dispatch_weight(_message_payload: &[u8]) -> bool {
		true
	}

	fn estimate_delivery_transaction(
		message_payload: &[u8],
		include_pay_dispatch_fee_cost: bool,
		message_dispatch_weight: Weight,
	) -> MessageTransaction<Weight> {
		let message_payload_len = u32::try_from(message_payload.len()).unwrap_or(u32::MAX);
		let extra_bytes_in_payload = Weight::from(message_payload_len)
			.saturating_sub(pallet_bridge_messages::EXPECTED_DEFAULT_MESSAGE_LENGTH.into());

		MessageTransaction {
			dispatch_weight: extra_bytes_in_payload
				.saturating_mul(bp_pass3dt::ADDITIONAL_MESSAGE_BYTE_DELIVERY_WEIGHT)
				.saturating_add(bp_pass3dt::DEFAULT_MESSAGE_DELIVERY_TX_WEIGHT)
				.saturating_sub(if include_pay_dispatch_fee_cost {
					0
				} else {
					bp_pass3dt::PAY_INBOUND_DISPATCH_FEE_WEIGHT
				})
				.saturating_add(message_dispatch_weight),
			size: message_payload_len
				.saturating_add(bp_pass3d::EXTRA_STORAGE_PROOF_SIZE)
				.saturating_add(bp_pass3dt::TX_EXTRA_BYTES),
		}
	}

	fn transaction_payment(transaction: MessageTransaction<Weight>) -> bp_pass3dt::Balance {
		// we don't have a direct access to the value of multiplier at Pass3dt chain
		// => it is a messages module parameter
		let multiplier = Pass3dtFeeMultiplier::get();
		// in our testnets, both per-byte fee and weight-to-fee are 1:1
		messages::transaction_payment(
			bp_pass3dt::BlockWeights::get().get(DispatchClass::Normal).base_extrinsic,
			1,
			multiplier,
			|weight| weight as _,
			transaction,
		)
	}
}

impl TargetHeaderChain<ToPass3dtMessagePayload, bp_pass3d::AccountId> for Pass3dt {
	type Error = &'static str;
	// The proof is:
	// - hash of the header this proof has been created with;
	// - the storage proof of one or several keys;
	// - id of the lane we prove state of.
	type MessagesDeliveryProof = ToPass3dtMessagesDeliveryProof;

	fn verify_message(payload: &ToPass3dtMessagePayload) -> Result<(), Self::Error> {
		messages::source::verify_chain_message::<WithPass3dtMessageBridge>(payload)
	}

	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<(LaneId, InboundLaneData<bp_pass3d::AccountId>), Self::Error> {
		messages::source::verify_messages_delivery_proof::<
			WithPass3dtMessageBridge,
			Runtime,
			crate::Pass3dtGrandpaInstance,
		>(proof)
	}
}

impl SourceHeaderChain<bp_pass3dt::Balance> for Pass3dt {
	type Error = &'static str;
	// The proof is:
	// - hash of the header this proof has been created with;
	// - the storage proof of one or several keys;
	// - id of the lane we prove messages for;
	// - inclusive range of messages nonces that are proved.
	type MessagesProof = FromPass3dtMessagesProof;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<ProvedMessages<Message<bp_pass3dt::Balance>>, Self::Error> {
		messages::target::verify_messages_proof::<
			WithPass3dtMessageBridge,
			Runtime,
			crate::Pass3dtGrandpaInstance,
		>(proof, messages_count)
	}
}

impl SenderOrigin<crate::AccountId> for crate::Origin {
	fn linked_account(&self) -> Option<crate::AccountId> {
		// XCM deals wit fees in our deployments
		None
	}
}

/// Pass3d -> Pass3dt message lane pallet parameters.
#[derive(RuntimeDebug, Clone, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub enum Pass3dToPass3dtMessagesParameter {
	/// The conversion formula we use is: `Pass3dTokens = Pass3dtTokens * conversion_rate`.
	Pass3dtToPass3dConversionRate(FixedU128),
}

impl MessagesParameter for Pass3dToPass3dtMessagesParameter {
	fn save(&self) {
		match *self {
			Pass3dToPass3dtMessagesParameter::Pass3dtToPass3dConversionRate(ref conversion_rate) =>
				Pass3dtToPass3dConversionRate::set(conversion_rate),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{DbWeight, Pass3dtGrandpaInstance, Runtime, WithPass3dtMessagesInstance};
	use bp_runtime::Chain;
	use bridge_runtime_common::{
		assert_complete_bridge_types,
		integrity::{
			assert_complete_bridge_constants, AssertBridgeMessagesPalletConstants,
			AssertBridgePalletNames, AssertChainConstants, AssertCompleteBridgeConstants,
		},
	};

	#[test]
	fn ensure_pass3d_message_lane_weights_are_correct() {
		type Weights = pallet_bridge_messages::weights::Pass3dtWeight<Runtime>;

		pallet_bridge_messages::ensure_weights_are_correct::<Weights>(
			bp_pass3d::DEFAULT_MESSAGE_DELIVERY_TX_WEIGHT,
			bp_pass3d::ADDITIONAL_MESSAGE_BYTE_DELIVERY_WEIGHT,
			bp_pass3d::MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT,
			bp_pass3d::PAY_INBOUND_DISPATCH_FEE_WEIGHT,
			DbWeight::get(),
		);

		let max_incoming_message_proof_size = bp_pass3dt::EXTRA_STORAGE_PROOF_SIZE.saturating_add(
			messages::target::maximal_incoming_message_size(bp_pass3d::Pass3d::max_extrinsic_size()),
		);
		pallet_bridge_messages::ensure_able_to_receive_message::<Weights>(
			bp_pass3d::Pass3d::max_extrinsic_size(),
			bp_pass3d::Pass3d::max_extrinsic_weight(),
			max_incoming_message_proof_size,
			messages::target::maximal_incoming_message_dispatch_weight(
				bp_pass3d::Pass3d::max_extrinsic_weight(),
			),
		);

		let max_incoming_inbound_lane_data_proof_size =
			bp_messages::InboundLaneData::<()>::encoded_size_hint_u32(
				bp_pass3d::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX as _,
				bp_pass3d::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX as _,
			);
		pallet_bridge_messages::ensure_able_to_receive_confirmation::<Weights>(
			bp_pass3d::Pass3d::max_extrinsic_size(),
			bp_pass3d::Pass3d::max_extrinsic_weight(),
			max_incoming_inbound_lane_data_proof_size,
			bp_pass3d::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX,
			bp_pass3d::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX,
			DbWeight::get(),
		);
	}

	#[test]
	fn ensure_bridge_integrity() {
		assert_complete_bridge_types!(
			runtime: Runtime,
			with_bridged_chain_grandpa_instance: Pass3dtGrandpaInstance,
			with_bridged_chain_messages_instance: WithPass3dtMessagesInstance,
			bridge: WithPass3dtMessageBridge,
			this_chain: bp_pass3d::Pass3d,
			bridged_chain: bp_pass3dt::Pass3dt,
		);

		assert_complete_bridge_constants::<
			Runtime,
			Pass3dtGrandpaInstance,
			WithPass3dtMessagesInstance,
			WithPass3dtMessageBridge,
			bp_pass3d::Pass3d,
		>(AssertCompleteBridgeConstants {
			this_chain_constants: AssertChainConstants {
				block_length: bp_pass3d::BlockLength::get(),
				block_weights: bp_pass3d::BlockWeights::get(),
			},
			messages_pallet_constants: AssertBridgeMessagesPalletConstants {
				max_unrewarded_relayers_in_bridged_confirmation_tx:
					bp_pass3dt::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX,
				max_unconfirmed_messages_in_bridged_confirmation_tx:
					bp_pass3dt::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX,
				bridged_chain_id: bp_runtime::PASS3DT_CHAIN_ID,
			},
			pallet_names: AssertBridgePalletNames {
				with_this_chain_messages_pallet_name: bp_pass3d::WITH_PASS3D_MESSAGES_PALLET_NAME,
				with_bridged_chain_grandpa_pallet_name: bp_pass3dt::WITH_PASS3DT_GRANDPA_PALLET_NAME,
				with_bridged_chain_messages_pallet_name:
					bp_pass3dt::WITH_PASS3DT_MESSAGES_PALLET_NAME,
			},
		});

		assert_eq!(
			Pass3dtToPass3dConversionRate::key().to_vec(),
			bp_runtime::storage_parameter_key(
				bp_pass3d::PASS3DT_TO_PASS3D_CONVERSION_RATE_PARAMETER_NAME
			)
			.0,
		);
	}
}
