#!/bin/bash

# THIS SCRIPT IS NOT INTENDED FOR USE IN PRODUCTION ENVIRONMENT
#
# This scripts periodically calls the Substrate relay binary to generate messages. These messages
# are sent from the Rialto network to the Millau network.

set -eu

# Max delay before submitting transactions (s)
MAX_SUBMIT_DELAY_S=${MSG_EXCHANGE_GEN_MAX_SUBMIT_DELAY_S:-30}
MESSAGE_LANE=${MSG_EXCHANGE_GEN_LANE:-00000000}
SECONDARY_MESSAGE_LANE=${MSG_EXCHANGE_GEN_SECONDARY_LANE}
MAX_UNCONFIRMED_MESSAGES_AT_INBOUND_LANE=1024

SHARED_CMD="/home/user/substrate-relay send-message rialto-to-millau"
SHARED_HOST="--source-host rialto-node-bob --source-port 9944"
SOURCE_SIGNER="--source-signer //Millau.MessagesSender"

SEND_MESSAGE="$SHARED_CMD $SHARED_HOST $SOURCE_SIGNER"

SOURCE_CHAIN="Rialto"
TARGET_CHAIN="Millau"
EXTRA_ARGS="--use-xcm-pallet"
REGULAR_PAYLOAD="020419ac"
BATCH_PAYLOAD="020419ac"

source /common/generate_messages.sh
