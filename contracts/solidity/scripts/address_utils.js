const { ethers, network } = require("hardhat");
const { readFileSync } = require('fs');

async function attachContract(name) {
  const filename = `../../public/${network.name}.json`;
  const obj = JSON.parse(readFileSync(filename, 'utf8'));
  const address = obj[name];

  const C = await ethers.getContractFactory(name);
  return await C.attach(address);
}

function sleep() {
  let waiting = 20000; // 20s
  if (network.name == "localhost") {
    waiting = 1000;
  }
  return new Promise(resolve => setTimeout(resolve, waiting));
}

module.exports = {
  attachContract,
  sleep
};
