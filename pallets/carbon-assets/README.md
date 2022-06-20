# Carbon Assets Pallet

Module for tokenization of carbon units from external registry. (Based on Parity [Asset pallet](https://github.com/paritytech/substrate/tree/polkadot-v0.9.23/frame/assets#assets-module) )

## Overview

The Carbon Assets module provides functionality for tokenization on Carbon Units from external registry. The current edition needs a manager who verifies the tokenization.

### Terminology

* **Custodian:** The Evercity manager. Only custodian can mint created carbon asset. Can be set in Genesis Config or by Sudo `set_custodian`.
* **Carbon Asset burning:** Burn of tokenized carbon asset. The owner recieves Burn Certificate.
* **BurnCertificate:**  The storage of amount of carbon assets burned per `AccountId` per `AssetId`. 

### Tokenization flow
1. User creates a carbon asset via `create` extrinsic. The name of the asset is generated. Asset decimals are set to 9.
2. User goes to external registry and buy and retire/transfer the asset with the generated name. User recieves some kind of public serial number of retirement.
3. User updates metadata of the asset via `set_project_data` extrinsic. User should include the serial number from previous step, amount of carbon units and some project information and store that on ipfs. The metadata updated with `url` and ipfs link `data_ipfs`.
4. Custodian verifies all data via link from previous step and `mint` carbon assets to user's account. 
5. User can burn carbon assets that they have (that is what carbon assets are made for) via `self_burn` extrinsic. Then user recieves BurnCertificate. User can burn particular carbon asset many times - all changes sum up in BurnCertificate.

## Interface

### Dispatchable Functions

Please refer to the `#[pallet::call]` for documentation on each function.

### Prerequisites

Add Carbon Assets Module to your Cargo.toml dependencies.

```toml
[dependencies]
pallet-carbon-assets = { version = "0.1.0", default-features = false, git = "https://github.com/EvercityEcosystem/carbon-assets.git" }
```
Also you need some source of `Randomness`, for example `pallet_randomness_collective_flip`.

### Configuration

Configure `construct_runtime!` in runtime/src/lib.rs

```rust
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip,
        ...
		CarbonAssets: pallet_carbon_assets,
	}
);
```
Configure `pallet_carbon_assets::Config`
```rust
parameter_types! {
	pub const CarbonAssetDeposit: Balance = 0;
	pub const CarbonAssetAccountDeposit: Balance = 0;
	pub const CarbonMetadataDepositBase: Balance = 0;
	pub const CarbonMetadataDepositPerByte: Balance = 0;
	pub const CarbonApprovalDeposit: Balance = 0;
	pub const CarbonStringLimit: u32 = 50;
}

pub use pallet_carbon_assets;
impl pallet_carbon_assets::Config for Runtime {
	type Event = Event;
	type Balance = u128;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type AssetDeposit = CarbonAssetDeposit;
	type AssetAccountDeposit = CarbonAssetAccountDeposit;
	type MetadataDepositBase = CarbonMetadataDepositBase;
	type MetadataDepositPerByte = CarbonMetadataDepositPerByte;
	type ApprovalDeposit = CarbonApprovalDeposit;
	type StringLimit = CarbonStringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = pallet_carbon_assets::weights::SubstrateWeight<Runtime>;
	type Randomness = RandomnessCollectiveFlip;
}
```

## Assumptions

Below are assumptions that must be held when using this module.  If any of
them are violated, the behavior of this module is undefined.

* The total count of assets should be less than
  `u64::MAX`.

## Related Modules

* [`System`](https://docs.rs/frame-system/latest/frame_system/)
* [`Support`](https://docs.rs/frame-support/latest/frame_support/)

License: Apache-2.0
