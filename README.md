## Bilateral Exchange Smart Contract

This a CosmWasm smart contract that provides the bilateral exchange of `provenance` `markers`.

## Build

_Make sure $PIO_HOME is set_

Compile and install

```bash
make && make install
```

## Quickstart

```bash
git clone git@github.com:provenance-io/provenance.git
git clone git@github.com:provenance-io/bilateral-exchange.git

cp bilateral-exchange/examples/bilateral.fish bilateral-exchange/examples/create-base.fish provenance
cd bilateral-exchange
make
cp artifacts/bilateral_exchange.wasm ../provenance
cd ../provenance
./create-base.fish
./bilateral.fish
```

## Example Usage

_NOTE: Address bech32 values and other params may vary._

0. Configure the following:
    1. Accounts:
        - Asker
        - Buyer
    1. Markers:
        - Base
        - Quote

0. Store the `bilateral-exchange` WASM:
    ```bash
    build/provenanced tx wasm store bilateral_exchange.wasm \
        -t \
        --source "https://github.com/provenance-io/bilateral_exchange" \
        --builder "cosmwasm/rust-optimizer:0.10.7" \
        --from validator \
        --keyring-backend test \
        --home build/run/provenanced \
        --chain-id testing \
        --gas auto \
        --fees 40000nhash \
        --broadcast-mode block \
        --yes | jq;
    ```
   
0. Instantiate the contract, binding the name `bilateral-ex.sc.pb` to the contract address:
    ```bash
    build/provenanced tx wasm instantiate 1 '{"bind_name":"bilateral-ex.sc","contract_name":"bilateral-ex"}' \
        -t \
        --admin (build/provenanced keys show -ta validator --home build/run/provenanced --keyring-backend test) \
        --from validator \
        --keyring-backend test \
        --home build/run/provenanced \
        --chain-id testing \
        --label ats-gme-usd \
        --gas auto \
        --gas-adjustment 1.4 \
        --fees 7000nhash \
        --broadcast-mode block \
        --yes | jq
    ```

0. Create an `ask` order:

    _NOTE: Replace `M2` with the `ask` base marker. Replace `M1_AMT` and `M1_DENOM` with quote marker_
   
    _NOTE++: The json data '{"create_ask":{}}' represents the action and additional data to pass into the smart contract, not the actual ask base. That is the `--amount` option._
    
    ```bash
    build/provenanced tx wasm execute tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
        '{"create_ask":{"id":"ask_id", "quote":[{"amount":"M1_AMT", "denom":"M1_DENOM"}]}}' \
        -t \
        --amount M2 \
        --from (build/provenanced keys show -ta seller --home build/run/provenanced --keyring-backend test) \
        --keyring-backend test \
        --home build/run/provenanced \
        --chain-id testing \
        --gas auto \
        --gas-adjustment 1.4 \
        --fees 5000nhash \
        --broadcast-mode block \
        --yes | jq
    ```

0. Create a `bid` order:

    _NOTE: Replace `M1` with the `bid` quote marker. Replace `M2_AMT` and `M2_DENOM` with base marker_
    
    _NOTE++: The json data '{"create_bid":{}}' represents the action and additional data to pass into the smart contract, not the actual bid quote. That is the `--amount` option._
    ```bash
    build/provenanced tx wasm execute tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
        '{"create_bid":{"id":"bid_id", "base":[{"amount":"M2_AMT", "denom":"M2_DENOM"}]}}' \
        --amount M1 \
        --from (build/provenanced keys show -ta buyer --home build/run/provenanced --keyring-backend test) \
        --keyring-backend test \
        --home build/run/provenanced \
        --chain-id testing \
        --gas auto \
        --gas-adjustment 1.4 \
        --fees 5000nhash \
        --broadcast-mode block \
        --yes \
        --testnet | jq
    ```

0. Match and execute the ask and bid orders.
   ```bash
    build/provenanced tx wasm execute tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
        '{"execute_match":{"ask_id":"ask_id", "bid_id":"bid_id"}}' \
        --from validator \
        --keyring-backend test \
        --home build/run/provenanced \
        --chain-id testing \
        --gas auto \
        --gas-adjustment 1.4 \
        --fees 6000nhash \
        --broadcast-mode block \
        --yes \
        --testnet | jq
    ```

## Other actions

Cancel the contract.

```bash
build/provenanced tx wasm execute tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
    '{"cancel_ask":{"id":"ask_id"}}' \
    -t \
    --from (build/provenanced keys show -ta seller --home build/run/provenanced --keyring-backend test) \
    --keyring-backend test \
    --home build/run/provenanced \
    --chain-id testing \
    --gas auto \
    --gas-adjustment 1.4 \
    --fees 5000nhash \
    --broadcast-mode block \
    --yes | jq
```

Query for ask order information:
```bash
provenanced query wasm contract-state smart tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
  '{"get_ask":{"id":"ask_id"}}' \
  --testnet
```

Query for bid order information:
```bash
provenanced query wasm contract-state smart tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
  '{"get_bid":{"id":"bid_id"}}' \
  --testnet
```

Query for contract instance information
```bash
provenanced query wasm contract-state smart tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
  '{"get_contract_info":{}}' \
  --testnet
```