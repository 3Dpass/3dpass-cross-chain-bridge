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

//! Pass3dt-to-Pass3d messages sync entrypoint.

use messages_relay::relay_strategy::MixStrategy;
use relay_pass3dt_client::Pass3dt;
use relay_pass3d_client::Pass3d;
use substrate_relay_helper::messages_lane::{
	DirectReceiveMessagesDeliveryProofCallBuilder, DirectReceiveMessagesProofCallBuilder,
	SubstrateMessageLane,
};

/// Description of Pass3dt -> Pass3d messages bridge.
#[derive(Clone, Debug)]
pub struct Pass3dtMessagesToPass3d;
substrate_relay_helper::generate_direct_update_conversion_rate_call_builder!(
	Pass3dt,
	Pass3dtMessagesToPass3dUpdateConversionRateCallBuilder,
	pass3dt_runtime::Runtime,
	pass3dt_runtime::WithPass3dMessagesInstance,
	pass3dt_runtime::pass3d_messages::Pass3dtToPass3dMessagesParameter::Pass3dToPass3dtConversionRate
);

impl SubstrateMessageLane for Pass3dtMessagesToPass3d {
	const SOURCE_TO_TARGET_CONVERSION_RATE_PARAMETER_NAME: Option<&'static str> =
		Some(bp_pass3d::PASS3DT_TO_PASS3D_CONVERSION_RATE_PARAMETER_NAME);
	const TARGET_TO_SOURCE_CONVERSION_RATE_PARAMETER_NAME: Option<&'static str> =
		Some(bp_pass3dt::PASS3D_TO_PASS3DT_CONVERSION_RATE_PARAMETER_NAME);

	const SOURCE_FEE_MULTIPLIER_PARAMETER_NAME: Option<&'static str> = None;
	const TARGET_FEE_MULTIPLIER_PARAMETER_NAME: Option<&'static str> = None;
	const AT_SOURCE_TRANSACTION_PAYMENT_PALLET_NAME: Option<&'static str> = None;
	const AT_TARGET_TRANSACTION_PAYMENT_PALLET_NAME: Option<&'static str> = None;

	type SourceChain = Pass3dt;
	type TargetChain = Pass3d;

	type SourceTransactionSignScheme = Pass3dt;
	type TargetTransactionSignScheme = Pass3d;

	type ReceiveMessagesProofCallBuilder = DirectReceiveMessagesProofCallBuilder<
		Self,
		pass3d_runtime::Runtime,
		pass3d_runtime::WithPass3dtMessagesInstance,
	>;
	type ReceiveMessagesDeliveryProofCallBuilder = DirectReceiveMessagesDeliveryProofCallBuilder<
		Self,
		pass3dt_runtime::Runtime,
		pass3dt_runtime::WithPass3dMessagesInstance,
	>;

	type TargetToSourceChainConversionRateUpdateBuilder =
		Pass3dtMessagesToPass3dUpdateConversionRateCallBuilder;

	type RelayStrategy = MixStrategy;
}
