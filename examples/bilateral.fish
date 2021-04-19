#!/usr/bin/env fish

printf "\n...buyer balance...\n";
build/provenanced query bank balances (build/provenanced keys show buyer -at --home build/run/provenanced --keyring-backend test) \
  -t \
  --home build/run/provenanced;

printf "\n...seller balance...\n";
build/provenanced query bank balances (build/provenanced keys show seller -at --home build/run/provenanced --keyring-backend test) \
-t \
--home build/run/provenanced;

printf "\n...contract balance...\n";
build/provenanced query bank balances tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
-t \
--home build/run/provenanced;

printf "\n...store wasm...\n";
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

printf "\n...instantiate contract...\n";
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

printf "\n...seller creating ask...\n";
build/provenanced tx wasm execute tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
    '{"create_ask":{"id":"ask_id", "quote":[{"amount":"8", "denom":"usd"}]}}' \
    -t \
    --amount 1gme \
    --from (build/provenanced keys show -ta seller --home build/run/provenanced --keyring-backend test) \
    --keyring-backend test \
    --home build/run/provenanced \
    --chain-id testing \
    --gas auto \
    --gas-adjustment 1.4 \
    --fees 5000nhash \
    --broadcast-mode block \
    --yes | jq

printf "\n...buyer creating bid...\n";
build/provenanced tx wasm execute tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
    '{"create_bid":{"id":"bid_id", "base":[{"amount":"1", "denom":"gme"}]}}' \
    --amount 8usd \
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

printf "\n...buyer balance...\n";
build/provenanced query bank balances (build/provenanced keys show buyer -at --home build/run/provenanced --keyring-backend test) \
  -t \
  --home build/run/provenanced;

printf "\n...seller balance...\n";
build/provenanced query bank balances (build/provenanced keys show seller -at --home build/run/provenanced --keyring-backend test) \
-t \
--home build/run/provenanced;

printf "\n...contract balance...\n";
build/provenanced query bank balances tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
-t \
--home build/run/provenanced;

printf "\n...ask order info...\n";
build/provenanced query wasm contract-state smart tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
  '{"get_ask":{"id":"ask_id"}}' \
  --ascii \
  --testnet

printf "\n...bid order info...\n";
build/provenanced query wasm contract-state smart tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
  '{"get_bid":{"id":"bid_id"}}' \
  --ascii \
  --testnet

printf "\n...contract info...\n";
build/provenanced query wasm contract-state smart \
    tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
    '{"get_contract_info":{}}' --testnet

printf "\n...executing match...\n";
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

printf "\n...buyer balance...\n";
build/provenanced query bank balances (build/provenanced keys show buyer -at --home build/run/provenanced --keyring-backend test) \
  -t \
  --home build/run/provenanced;

printf "\n...seller balance...\n";
build/provenanced query bank balances (build/provenanced keys show seller -at --home build/run/provenanced --keyring-backend test) \
-t \
--home build/run/provenanced;

printf "\n...contract balance...\n";
build/provenanced query bank balances tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
-t \
--home build/run/provenanced;

printf "\n...seller creating ask...\n";
build/provenanced tx wasm execute tp18vd8fpwxzck93qlwghaj6arh4p7c5n89x8kskz \
    '{"create_ask":{"id":"ask_id", "quote":[{"amount":"8", "denom":"usd"}]}}' \
    -t \
    --amount 1gme \
    --from (build/provenanced keys show -ta seller --home build/run/provenanced --keyring-backend test) \
    --keyring-backend test \
    --home build/run/provenanced \
    --chain-id testing \
    --gas auto \
    --gas-adjustment 1.4 \
    --fees 5000nhash \
    --broadcast-mode block \
    --yes | jq

printf "\n...seller canceling ask...\n";
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
