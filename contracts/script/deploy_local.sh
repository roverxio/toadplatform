#!/usr/bin/env bash
anvil --mnemonic 'test test test test test test test test test test test junk' >/dev/null 2>&1 &

until curl -s http://localhost:8545 >/dev/null;
do
    sleep 1
done

# shellcheck disable=SC2046
var=$(dirname "$(dirname $(realpath "$0"))")
cd "${var}" || exit
source .env
CREATE2_ADDRESS="0x4e59b44847b379578588920cA78FbF26c0B4956C"
curl http://localhost:8545 -X POST -H 'Content-Type: application/json' --data "{\"jsonrpc\":\"2.0\", \"id\":1, \"method\": \"anvil_setCode\", \"params\": [\"$CREATE2_ADDRESS\", \"0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf3\"]}" >/dev/null 2>&1;

var=$(forge script script/DeployLocal.s.sol:DeployLocal --rpc-url "${RPC_URL}" --broadcast 2>&1)

while IFS= read -r line; do
  if [[ "$line" == "Error:"* ]]; then
      echo -e "\033[1;31mError\033[0;31m: Contracts already deployed.\n\033[1;37mHINT\033[0;37m: Kill anvil and run the script again to redeploy the contracts"
      exit 1
  fi
done <<< "${var}"

while IFS= read -r line; do
  if [[ "$line" == *"addr="* ]]; then
      echo -e "\033[0;32m$line"
  fi
done <<< "${var}"