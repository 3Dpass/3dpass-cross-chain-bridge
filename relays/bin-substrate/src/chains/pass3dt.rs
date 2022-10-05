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

//! Pass3dt chain specification for CLI.

use crate::cli::{
	bridge,
	encode_message::{CliEncodeMessage, RawMessage},
	CliChain,
};
use bp_messages::LaneId;
use bp_runtime::EncodedOrDecodedCall;
use relay_pass3dt_client::Pass3dt;
use relay_substrate_client::BalanceOf;
use sp_version::RuntimeVersion;
use xcm::latest::prelude::*;

impl CliEncodeMessage for Pass3dt {
	fn encode_send_xcm(
		message: xcm::VersionedXcm<()>,
		bridge_instance_index: u8,
	) -> anyhow::Result<EncodedOrDecodedCall<Self::Call>> {
		let dest = match bridge_instance_index {
			bridge::PASS3DT_TO_PASS3D_INDEX =>
				(Parent, X1(GlobalConsensus(pass3dt_runtime::xcm_config::Pass3dNetwork::get()))),
			_ => anyhow::bail!(
				"Unsupported target bridge pallet with instance index: {}",
				bridge_instance_index
			),
		};

		Ok(pass3dt_runtime::Call::XcmPallet(pass3dt_runtime::XcmCall::send {
			dest: Box::new(dest.into()),
			message: Box::new(message),
		})
		.into())
	}

	fn encode_send_message_call(
		lane: LaneId,
		payload: RawMessage,
		fee: BalanceOf<Self>,
		bridge_instance_index: u8,
	) -> anyhow::Result<EncodedOrDecodedCall<Self::Call>> {
		Ok(match bridge_instance_index {
			bridge::PASS3DT_TO_PASS3D_INDEX => pass3dt_runtime::Call::BridgePass3dMessages(
				pass3dt_runtime::MessagesCall::send_message {
					lane_id: lane,
					payload,
					delivery_and_dispatch_fee: fee,
				},
			)
			.into(),
			_ => anyhow::bail!(
				"Unsupported target bridge pallet with instance index: {}",
				bridge_instance_index
			),
		})
	}
}

impl CliChain for Pass3dt {
	const RUNTIME_VERSION: Option<RuntimeVersion> = Some(pass3dt_runtime::VERSION);

	type KeyPair = sp_core::sr25519::Pair;
	type MessagePayload = Vec<u8>;

	fn ss58_format() -> u16 {
		pass3dt_runtime::SS58Prefix::get() as u16
	}
}
