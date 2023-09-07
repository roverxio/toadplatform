#!/usr/bin/env bash
anvil --mnemonic 'test test test test test test test test test test test junk' >/dev/null 2>&1 &

# loop to check if anvil has started
until curl -s http://localhost:8545 >/dev/null;
do
    sleep 1
done

# the following set of lines enable us to deploy the script from anywhere in the project
cd "$(dirname "$0")" || exit
cd "$(dirname "$(pwd)")" || exit
source .env
CREATE2_ADDRESS="0x4e59b44847b379578588920cA78FbF26c0B4956C" # a fixed address at which anvil checks for the create 2 factory
curl http://localhost:8545 -X POST -H 'Content-Type: application/json' --data "{\"jsonrpc\":\"2.0\", \"id\":1, \"method\": \"anvil_setCode\", \"params\": [\"$CREATE2_ADDRESS\", \"0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf3\"]}" >/dev/null 2>&1;

var=$(forge script script/DeployLocal.s.sol:DeployLocal --rpc-url "${RPC_URL}" --broadcast 2>&1) # store the outputs/logs of forge scrip to the variable

# loop to check for errors
while IFS= read -r line; do
  if [[ "$line" == "Error:"* ]]; then
      echo -e "\033[1;31mError\033[0;31m: Contracts already deployed.\n\033[1;37mHINT\033[0;37m: Kill anvil using and run the script again to redeploy the contracts\nTry:\t\t\033[0;0m pkill -f anvil"
      exit 1
  fi
done <<< "${var}"

echo -e "\033[0;32m== Key= $PRIVATE_KEY"
# loop to look for and output the relevant addresses that are created/used in forge script
while IFS= read -r line; do
  if [[ "$line" == *"addr="* ]]; then
      echo -e "\033[0;32m$line"
  fi
done <<< "${var}"