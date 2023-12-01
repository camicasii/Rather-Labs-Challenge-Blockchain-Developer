import { ApiNetworkProvider } from "@multiversx/sdk-network-providers";
import {
  AbiRegistry,
  Address,
  Account,
  SmartContract,
  AddressValue,
  BigUIntType,
  BinaryCodec,
  ResultsParser,
} from "@multiversx/sdk-core";
import { readFileSync } from "fs";
import path from "path";

const apiNetworkProvider = new ApiNetworkProvider(
  "https://devnet-api.multiversx.com"
);
let existingContractAddress = Address.fromBech32(
  "erd1qqqqqqqqqqqqqpgqlecks6h4lwk45fa8ush5hpzhmcf53hthetes4e8asz"
);
const addressOfAlice_ =
  "erd1hsjenjp8rhl8rdy7tuxftl26rrd4x4rceak78g0xdfykvvg8etesk45cku";

async function networkConfig() {
  const networkConfig_ = await apiNetworkProvider.getNetworkConfig();
  console.log(networkConfig_.MinGasLimit);
  console.log(networkConfig_.ChainID);
  return networkConfig_;
}

async function getBAlance(params: string) {
  const addressOfAlice = Address.fromString(params);
  const alice = new Account(addressOfAlice);
  const aliceOnNetwork = await apiNetworkProvider.getAccount(addressOfAlice);
  alice.update(aliceOnNetwork);
  console.log("Nonce:", alice.nonce);
  console.log("Balance:", alice.balance.toString());
}

function parserData(queryResponse: any, getStakingGlobal: any) {
  const res = new ResultsParser().parseQueryResponse(
    queryResponse,
    getStakingGlobal
  );
  return res.values[0].valueOf();
}
async function name() {
  let abiJson = readFileSync(
    path.join(
      __dirname,
      "../../staking-contract/output/staking-contract.abi.json"
    ),
    "utf-8"
  );
  let abiObj = JSON.parse(abiJson);
  let abiRegistry = AbiRegistry.create(abiObj);

  let legacyDelegationContract = new SmartContract({
    address: existingContractAddress,
    abi: abiRegistry,
  });

  const getStakingGlobal =
    legacyDelegationContract.getEndpoint("getStakingGlobal");
  const getStakingPosition =
    legacyDelegationContract.getEndpoint("getStakingPosition");

  const addressOfAlice = Address.fromString(addressOfAlice_);

  let query = legacyDelegationContract.createQuery({
    func: "getStakingPosition",
    args: [new AddressValue(addressOfAlice)],
  });

  let query2 = legacyDelegationContract.createQuery({
    func: "getStakingGlobal",
  });

  let queryResponse = await apiNetworkProvider.queryContract(query);
  let queryResponse2 = await apiNetworkProvider.queryContract(query2);

  // pub stake_amount: BigUint<M>,
  // pub total_invested: BigUint<M>,
  // pub total_withdrawn: BigUint<M>,
  // pub reward_per_second: BigUint<M>,
  // pub reward_per_block: BigUint<M>,

  const stakingGlobal = parserData(queryResponse2, getStakingGlobal);

  // // pub stake_amount: BigUint<M>,
  // // pub total_invested: BigUint<M>,
  // // pub total_withdrawn: BigUint<M>,
  // // pub last_action_block: u64,
  const stakingPosition = parserData(queryResponse, getStakingPosition);

  console.log("stakingGlobal", {
    stake_amount: stakingGlobal.stake_amount.toString(),
    total_invested: stakingGlobal.total_invested.toString(),
    total_withdrawn: stakingGlobal.total_withdrawn.toString(),
    reward_per_second: stakingGlobal.reward_per_second.toString(),
    reward_per_block: stakingGlobal.reward_per_block.toString(),
  });

  console.log("stakingPosition", {
    stake_amount: stakingPosition.stake_amount.toString(),
    total_invested: stakingPosition.total_invested.toString(),
    total_withdrawn: stakingPosition.total_withdrawn.toString(),
    last_action_block: stakingPosition.last_action_block.toString(),
  });
}
name();
