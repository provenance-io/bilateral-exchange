## Bilateral Exchange Smart Contract

This a CosmWasm smart contract that provides the bilateral exchange of `provenance` `markers`.

## Build

_Make sure $PIO_HOME is set_

Compile and install

```bash
make && make install
```

## Example Usage

_NOTE: Address bech32 values and other params may vary._

_Optionally create an unrestricted name binding parent for smart-contracts_
```bash
provenanced tx name bind \
    "sc" \
    (provenanced keys show -a node0 --home build/node0 --keyring-backend test --testnet) \
    "pb" \
    --restrict=false \
    --from node0 \
    --keyring-backend test \
    --home build/node0 \
    --chain-id chain-local \
    --fees 5000nhash \
    --broadcast-mode block \
    --yes \
    --testnet | jq
```

1. Store the `bilateral-exchange` WASM.

```bash
provenanced tx wasm store bilateral_exchange.wasm \
    --source "https://github.com/figuretechnologies/bilateral-exchange" \
    --builder "cosmwasm/rust-optimizer:0.10.7" \
    --from node0 \
    --keyring-backend test \
    --home build/node0 \
    --chain-id chain-local \
    --gas auto \
    --fees 25000nhash \
    --broadcast-mode block \
    --yes \
    --testnet | jq
```

2. Instantiate the contract, binding the name `bilateral-ex.sc.pb` to the contract address.

_NOTE: Replace `M1_AMT` and `M1_DENOM` with the `ask` marker. Replace `M2` with the `collateral` marker._

```bash
provenanced tx wasm instantiate 1 \
    '{"bind_name":"bilateral-ex.sc.pb","contract_name":"bilateral-ex"}' \
    --admin (provenanced keys show -a node0 --home build/node0 --keyring-backend test --testnet) \
    --from node0 \
    --keyring-backend test \
    --home build/node0 \
    --chain-id chain-local \
    --label bilateral-gme \
    --gas auto \
    --gas-adjustment 1.4 \
    --fees 5000nhash \
    --broadcast-mode block \
    --yes \
    --testnet | jq
```

3. Create an `ask` order.

_NOTE: Replace `M2` with the `ask` asset marker. Replace `M1_AMT` and `M1_DENOM` with ask price marker_

_NOTE++: The json data '{"create_ask":{}}' represents the action and additional data to pass into the smart contract, not the actual ask asset. That is the `--amount` option._

```bash
provenanced tx wasm execute \
    tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
    '{"create_ask":{"id":"ask_id", "price":[{"amount":"M1_AMT","denom":"M1_DENOM"}]}}' \
    --amount M2 \
    --from (provenanced keys show -a asker --home build/node0 --keyring-backend test --testnet) \
    --keyring-backend test \
    --home build/node0 \
    --chain-id chain-local \
    --gas auto \
    --gas-adjustment 1.4 \
    --fees 5000nhash \
    --broadcast-mode block \
    --yes \
    --testnet | jq
```

4. Create a `bid` order.

_NOTE: Replace `M1` with the `bid` price marker. Replace `M2_AMT` and `M2_DENOM` with bid asset marker_

_NOTE++: The json data '{"create_bid":{}}' represents the action and additional data to pass into the smart contract, not the actual ask asset. That is the `--amount` option._

```bash
provenanced tx wasm execute \
    tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
    '{"create_bid":{"id":"bid_id", "asset":[{"amount":"M1_AMT","denom":"M1_DENOM"}]}}' \
    --amount M1 \
    --from (provenanced keys show -a bidder --home build/node0 --keyring-backend test --testnet) \
    --keyring-backend test \
    --home build/node0 \
    --chain-id chain-local \
    --gas auto \
    --gas-adjustment 1.4 \
    --fees 5000nhash \
    --broadcast-mode block \
    --yes \
    --testnet | jq
```

4. Match and execute the ask and bid orders.

```bash
provenanced tx wasm execute \
    tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
    '{"execute":{"ask_id":"ask_id", "bid_id":"bid_id"}}' \
    --from node0 \
    --keyring-backend test \
    --home build/node0 \
    --chain-id chain-local \
    --gas auto \
    --gas-adjustment 1.4 \
    --fees 5000nhash \
    --broadcast-mode block \
    --yes \
    --testnet | jq  
```

## Other actions

Cancel the contract.

```bash
provenanced tx wasm execute \
  tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
  '{"cancel":{}}' \
  --from seller \
  --keyring-backend test \
  --home build/node0 \
  --chain-id chain-local \
  --gas auto \
  --gas-adjustment 1.4 \
  --fees 5000nhash \
  --broadcast-mode block \
  --yes \
  --testnet | jq
```

Query the state of the contract.

```bash
provenanced query wasm contract-state smart \
    tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz '{"query_state":{}}' --testnet
```