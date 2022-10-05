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

//! Pass3d-to-Pass3dt messages sync entrypoint.

use messages_relay::relay_strategy::MixStrategy;
use relay_pass3dt_client::Pass3dt;
use relay_pass3d_client::Pass3d;
use substrate_relay_helper::messages_lane::{
	DirectReceiveMessagesDeliveryProofCallBuilder, DirectReceiveMessagesProofCallBuilder,
	SubstrateMessageLane,
};

/// Description of Pass3d -> Pass3dt messages bridge.
#[derive(Clone, Debug)]
pub struct Pass3dMessagesToPass3dt;
substrate_relay_helper::generate_direct_update_conversion_rate_call_builder!(
	Pass3d,
	Pass3dMessagesToPass3dtUpdateConversionRateCallBuilder,
	pass3d_runtime::Runtime,
	pass3d_runtime::WithPass3dtMessagesInstance,
	pass3d_runtime::pass3dt_messages::Pass3dToPass3dtMessagesParameter::Pass3dtToPass3dConversionRate
);

impl SubstrateMessageLane for Pass3dMessagesToPass3dt {
	const SOURCE_TO_TARGET_CONVERSION_RATE_PARAMETER_NAME: Option<&'static str> =
		Some(bp_pass3dt::PASS3D_TO_PASS3DT_CONVERSION_RATE_PARAMETER_NAME);
	const TARGET_TO_SOURCE_CONVERSION_RATE_PARAMETER_NAME: Option<&'static str> =
		Some(bp_pass3d::PASS3DT_TO_PASS3D_CONVERSION_RATE_PARAMETER_NAME);

	const SOURCE_FEE_MULTIPLIER_PARAMETER_NAME: Option<&'static str> = None;
	const TARGET_FEE_MULTIPLIER_PARAMETER_NAME: Option<&'static str> = None;
	const AT_SOURCE_TRANSACTION_PAYMENT_PALLET_NAME: Option<&'static str> = None;
	const AT_TARGET_TRANSACTION_PAYMENT_PALLET_NAME: Option<&'static str> = None;

	type SourceChain = Pass3d;
	type TargetChain = Pass3dt;

	type SourceTransactionSignScheme = Pass3d;
	type TargetTransactionSignScheme = Pass3dt;

	type ReceiveMessagesProofCallBuilder = DirectReceiveMessagesProofCallBuilder<
		Self,
		pass3dt_runtime::Runtime,
		pass3dt_runtime::WithPass3dMessagesInstance,
	>;
	type ReceiveMessagesDeliveryProofCallBuilder = DirectReceiveMessagesDeliveryProofCallBuilder<
		Self,
		pass3d_runtime::Runtime,
		pass3d_runtime::WithPass3dtMessagesInstance,
	>;

	type TargetToSourceChainConversionRateUpdateBuilder =
		Pass3dMessagesToPass3dtUpdateConversionRateCallBuilder;

	type RelayStrategy = MixStrategy;
}
