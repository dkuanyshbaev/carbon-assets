# Carbon Assets Pallet

Module for tokenization of carbon units from the external registry. (Based on Parity [Asset pallet](https://github.com/paritytech/substrate/tree/polkadot-v0.9.23/frame/assets#assets-module) )

## Overview

The Carbon Assets module provides functionality for the tokenization of Carbon Units from the external registry. The current edition needs a manager who verifies the tokenization.

### Terminology

* **Custodian:** The Evercity manager. Only a custodian can mint created carbon asset. Can be set in Genesis Config or by Sudo `set_custodian`.
* **Carbon Asset burning:** Burn of tokenized carbon asset. The owner receives Burn Certificate.
* **BurnCertificate:**  The storage of amount of carbon assets burned per `AccountId` per `AssetId`. 

### Basic flow configuration

1. It's necessary to setup custodian for minting assets. Use sudo `set_custodian` extrinsic to some address
2. Don't forget to replenish balances of addresses which will hold assets (even those where you're going to transfer to)

### Tokenization flow
1. User creates a carbon asset via `create` extrinsic. The user sets a name and a symbol of the asset. Asset decimals are set to 9. `AssetId` is generated.
2. User goes to the external registry and buys and retires/transfers the asset with the generated `AssetId` (and maybe name too). The user receives some kind of public serial number of retirement.
3. User updates metadata of the asset via `set_project_data` extrinsic. The user should include the serial number from the previous step, and some project information and store that on ipfs. The metadata is updated with `url` and ipfs link `data_ipfs`.
4. Custodian verifies all data via the link from the previous step and `mint` carbon assets to the user's account. 
5. The user can burn carbon assets that they have (that is what carbon assets are made for) via `self_burn` extrinsic. Then user receives a BurnCertificate. The user can burn a particular carbon asset many times - all changes sum up in the BurnCertificate. The Custodian also can burn carbon assets for the user via `burn` extrinsic. The user also receives a BurnCertificate.

## Interface

### Dispatchable Functions

Please refer to the `#[pallet::call]` for documentation on each function.

### Prerequisites

Add Carbon Assets Module to your Cargo.toml dependencies.

```toml
[dependencies]
pallet-carbon-assets = { version = "0.2.0", default-features = false, git = "https://github.com/EvercityEcosystem/carbon-assets.git" }
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
	pub const CarbonStringLimit: u32 = 140;
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
