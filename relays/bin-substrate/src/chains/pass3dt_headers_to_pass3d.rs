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

//! Pass3d-to-Pass3d headers sync entrypoint.

use crate::cli::bridge::{CliBridgeBase, MessagesCliBridge, RelayToRelayHeadersCliBridge};
use substrate_relay_helper::finality::{
	engine::Grandpa as GrandpaFinalityEngine, DirectSubmitGrandpaFinalityProofCallBuilder,
	SubstrateFinalitySyncPipeline,
};

/// Description of Pass3d -> Pass3d finalized headers bridge.
#[derive(Clone, Debug)]
pub struct Pass3dtFinalityToPass3d;

impl SubstrateFinalitySyncPipeline for Pass3dtFinalityToPass3d {
	type SourceChain = relay_pass3dt_client::Pass3dt;
	type TargetChain = relay_pass3d_client::Pass3d;

	type FinalityEngine = GrandpaFinalityEngine<Self::SourceChain>;
	type SubmitFinalityProofCallBuilder = DirectSubmitGrandpaFinalityProofCallBuilder<
		Self,
		pass3d_runtime::Runtime,
		pass3d_runtime::Pass3dtGrandpaInstance,
	>;
	type TransactionSignScheme = relay_pass3d_client::Pass3d;
}

//// `Pass3d` to `Pass3d` bridge definition.
pub struct Pass3dtToPass3dCliBridge {}

impl CliBridgeBase for Pass3dtToPass3dCliBridge {
	type Source = relay_pass3dt_client::Pass3dt;
	type Target = relay_pass3d_client::Pass3d;
}

impl RelayToRelayHeadersCliBridge for Pass3dtToPass3dCliBridge {
	type Finality = Pass3dtFinalityToPass3d;
}

impl MessagesCliBridge for Pass3dtToPass3dCliBridge {
	const ESTIMATE_MESSAGE_FEE_METHOD: &'static str =
		bp_pass3d::TO_PASS3D_ESTIMATE_MESSAGE_FEE_METHOD;
	type MessagesLane = crate::chains::pass3dt_messages_to_pass3d::Pass3dtMessagesToPass3d;
}
