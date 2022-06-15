# 1. Evercity Accounts Pallet

This repositary contains source code of blockchain node, which is a main part of Evercity's Accounts. The pallet has a purely technical purpose and is used for interaction with other pallets.

# 2. Evercity accounts main entities

Accounts pallet has several entities: 

### 2.1 AccountStruct 

Is the main entity for accounts, containing rolemask of account in pallet storage

### 2.2 RoleMask 

Each Evercity account can can accommodate one or more roles:

- MASTER: the administrative role that can assign roles to accounts
- CUSTODIAN:  the role which can mint and burn the main platform token. This role is assigned to the public account of the partner bank, which exchanges USD --> EVERUSD and EVERUSD --> USD.
- ISSUER: the role which can create bonds. An account with the ISSUER role issues a bond to fund a sustainability-aligned project. After receiving funds from the sale of Bond Units, the ISSUER undertakes to provide data on the impact of the project, which influences the coupon rate that should be paid to the investor. The ISSUER is obliged to replenish the bond balance with the amount necessary to cover its financial obligations.
- INVESTOR: accounts with the INVESTOR role use the EVERUSD token to buy Bond Units and sell them on the secondary market. Each billing period Investor receives a coupon income proportional to its balances of various Bond Units
- AUDITOR: these accounts check and confirm the environmental impact data sent by Issuer, as well as certify the documents uploaded to the platform
- MANAGER: the task of accounts with this role is to help Issuers work with projects, verify data and prepare documents
- IMPACT_REPORTER: send impact reports
- BOND_ARRANGER: This role regulates the launch of bonds to the market, making the final decision on whether the bond meets the requirements.
- CC_PROJECT_OWNER: the role which can create carbon projects, annual report and issue caebon credits
- CC_AUDITOR: the role to sign project documentation and annual reports according to carbon credits standard
- CC_STANDARD: the role to sign project documentation and annual reports according to carbon credits standard
- CC_INVESTOR: carbon credits investor
- CC_REGISTRY: the role to sign project documentation and annual reports according to carbon credits standard

# 3. Evercity Account pallet can do several things

- Set MASTER role on account
- Set any non-MASTER role
- Add additional non-MASTER role on account
- Withraw any non-MASTER role

# 4. Accounts documentation

### 4.1 Runtime methods

<!-- Methods of pallet-evercity are described in Rust documentation [here](http://51.15.47.43/doc/pallet_evercity/) [TEMP] -->

### 4.2 Build

```bash
git clone https://github.com/EvercityEcosystem/evercity-accounts
cd evercity-accounts
make build
```
### 4.3 Add to runtime cargo.toml

```toml
pallet-evercity-accounts = { default-features = false, version = '0.1.8', git = 'https://github.com/EvercityEcosystem/evercity-accounts' }
#...
[features]
default = ['std']

std = [
    #...
    'pallet-evercity-accounts/std',
    #...
]
```

### 4.4 Add to runtime constructing

```rust
pub use pallet_evercity_accounts;
impl pallet_evercity_accounts::Config for Runtime {
    type Event = Event;
}
...
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        ...
        EvercityAccounts: pallet_evercity_accounts::{ Module, Call, Storage, Config<T>, Event<T>},
        ...
    }
);
```

### 4.5 Modify chain spec, set GenesisConfig

With no predefined accounts (master account is set using `set_master()` extrinsic)

```rust
use node_template_runtime::EvercityAccountsConfig;
...
GenesisConfig {
...
	pallet_evercity_accounts: Some(EvercityAccountsConfig {
            // set roles for each pre-set accounts (set role)
            genesis_account_registry: Vec::new()
        }),
}
```

With predefined accounts

```rust
use node_template_runtime::EvercityAccountsConfig;
...

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    /// predefined evercity accounts with roles
    evercity_accounts: Vec<(AccountId, pallet_evercity_accounts::accounts::RoleMask)>,
    ...
) -> GenesisConfig {
    GenesisConfig {
        ...
         pallet_evercity_accounts: Some(EvercityAccountsConfig {
            // set roles for each pre-set accounts (set role)
            genesis_account_registry: evercity_accounts
                .iter()
                .map(|(acc, role)| {
                    (
                        acc.clone(),
                        AccountStruct {
                            roles: *role,
                            identity: 0,
                            create_time: 0,
                        },
                    )
                })
                .collect(),
        }),
    }
}
```

### 4.6 Run Unit Tests

```bash
make test
```

### 4.7 Launch linter

```bash
make lint
```
