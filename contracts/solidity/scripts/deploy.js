// We require the Hardhat Runtime Environment explicitly here. This is optional
// but useful for running the script in a standalone fashion through `node <script>`.
//
// You can also run a script with `npx hardhat run <script>`. If you do that, Hardhat
// will compile your contracts, add the Hardhat Runtime Environment's members to the
// global scope, and execute the script.
const { ethers, upgrades, network } = require("hardhat");
const { attachContract, sleep } = require("./address_utils.js");
const { writeFile } = require('fs');

async function deployContractWithProxy(name, params=[]) {
  const Factory = await ethers.getContractFactory(name);
  //  use upgradeable deploy, then contracts can be upgraded success, otherwise will get error about ERC 1967 proxy
  const contract = await upgrades.deployProxy(Factory, params);
  await contract.deployed();
  console.log(`${name} address: ${contract.address}`);

  return contract;
}

async function deployContract(name, params=[]) {
  const Factory = await ethers.getContractFactory(name);
  const contract = await Factory.deploy(...params);
  const address = await contract.getAddress();
  console.log(`${name} address: ${address}`);

  return address;
}

async function deploy() {
  // Zypher Game Coin & SimpleGame
  const token = await deployContract("Token", [1000000]);
  const game = await deployContract("SimpleGame", [token, 10000, 100, 4, 10000]);

  const addresses = {
    Token: token,
    RoomMarket: game,
    SimpleGame: game,
  };
  const filename = `../../public/${network.name}.json`;
  writeFile(
    filename,
    JSON.stringify(addresses, null, 2),
    function(err) {
      if (err) {
        console.log(err);
      }
    });
  console.log(`Save to ${filename}`);
}

async function main() {
  await deploy();
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
