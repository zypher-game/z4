require("@nomicfoundation/hardhat-toolbox");
require("hardhat-contract-sizer");
require("hardhat-gas-reporter");
require("dotenv").config();

const SECRET_KEY = process.env.SECRET_KEY ?? '';

// SAFE: Hardhat default sk
const LOCAL_SK1 = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  networks: {
    hardhat: {
      mining: {
        auto: true,
        interval: 2000
      },
    },
    localhost: {
      url: 'http://127.0.0.1:8545',
      accounts: [LOCAL_SK1]
    },
    holesky: {
      url: 'https://1rpc.io/holesky',
      accounts: [SECRET_KEY]
    },
    sepolia: {
      url: 'https://rpc.sepolia.org',
      accounts: [SECRET_KEY]
    },
    opbnbtestnet: {
      url: 'https://opbnb-testnet-rpc.bnbchain.org',
      accounts: [SECRET_KEY]
    },
  },
  solidity: {
    version: "0.8.20",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200
      }
    }
  },
  paths: {
    sources: "./contracts",
    tests: "./test",
    cache: './build/cache',
    artifacts: './build/artifacts',
  },
  contractSizer: {
    alphaSort: true,
    runOnCompile: true,
    disambiguatePaths: false,
  },
  mocha: {
    timeout: 200000
  },
  gasReporter: {
    currency: 'USD',
    enabled: true,
  },
  etherscan: {
    // Your API key for Etherscan
    // Obtain one at https://etherscan.io/
    apiKey: '<api-key>',
  }
};
