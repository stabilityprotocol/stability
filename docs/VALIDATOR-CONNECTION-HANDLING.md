# Validator Connection Handling

Stability is Proof of Reputation blockchain that means that it has a list of approved validators. Each validator would be responsible to mine a block in their assigned slot. The problem comes when one of the validators is offline, his slot would be skipped. This represent a problem since it could affect negatively to the actual block time of Stability.

## Solution

The solution is implemented in `pallet_validator_set` and it keeps track of mined blocks' author while is incharge of giving the `pallet_session` (which is the pallet that controls which validators will author in the each session) the validator list of the next session.

The criteria for a validator to be on the list of active validators is simple: It has to be online

### How is determined if a validator is offline?

A validator would be considered offline if they haven't mined a block in more than `MaxMissedEpochs`epochs. `MaxMissedEpochs`is an updateable value that currently is set at `5`.

### How could a validator get back online?

A validator that was removed from active validators list should submit a `pallet_validator_set::Call::add_validator_again` (unsigned extrinsic) to be included in the list again. This change won't reflect until two epochs after the extrinsic was emitted.

Note: For those running validators, there is no action needed in order to recover a validator from being offline since there exists an offline worker that would emit the needed extrinsic.
