# UPGRADE RUNTIME PROPOSAL PALLET

This pallet is used to update the runtime.

## EXTRINSICS

### propose_code

This extrinsic is used to propose a runtime WASM byte code. The custom `ControlOrigin` of the pallet must allow the origin. If a code is already proposed, it should be reverted before proposing another code.

The argument of this extrinsic:

- *code*: Runtime WASM bytecode proposed.


### set_block_application

This extrinsic is used to set the block to which the proposed code will be applied. The origin must be root.

The argument of this extrinsic:

- *block_number*: Block number to which the proposed code will be applied.

### reject_proposed_code


This extrinsic is used to reject the current proposed code. The origin must be root.


