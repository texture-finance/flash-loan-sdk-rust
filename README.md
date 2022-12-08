# Flash Loan program Rust SDK

## Functions
* ```available_liquidity``` - Returns maximum amount of tokens which could be flash borrowed from given reserve. Use this function when you have deserialized Reserve structure.
* ```available_liquidity_via_rpc``` -	Returns maximum amount of tokens which could be flash borrowed from given reserve. Use this function when you have reserve’s Pubkey and already inited RpcClient.
* ```flash_loan_fee``` -	Calculates total fees for flash borrow of specified amount Type of token to be borrowed is determined by reserve
* ```flash_loan_fee_via_rpc``` -	Calculates total fees for flash borrow of specified amount Type of token to be borrowed is determined by reserve Use this function when you have reserve’s Pubkey and already inited RpcClient.
* ```get_reserve``` -	Returns deserialized Reserve structure getting it from account specified by reserve_key via provided RpcClient
* ```flash_borrow``` -	Creates a ‘FlashBorrow’ instruction.
* ```flash_repay``` -	Creates a ‘FlashRepay’ instruction.

Usage example please see in ```examples/flash_loan_once.rs```

## Addresses
Program ID of Flash Loan contract on devnet and mainnet: F1aShdFVv12jar3oM2fi6SDqbefSnnCVRzaxbPH3you7
It is defined as FLASH_LOAN_ID constant in this SDK.

wSOL reserve on devnet: 9Wys2sCHcAGZm3jgSnfP8xyq1ZiK2qthQ4Ki5fSdkqP 


## Install and build
Clone the repo and then in the root source directory:
```shell
cargo build
```

Generate documentation:
```shell
cargo doc
```

## Run example
```shell
cargo run
```