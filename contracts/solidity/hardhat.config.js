require("@nomicfoundation/hardhat-toolbox");
require("@nomiclabs/hardhat-ethers");
require('@openzeppelin/hardhat-upgrades');
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
    mumbai: {
      url: 'https://rpc.ankr.com/polygon_mumbai',
      accounts: [SECRET_KEY]
    },
    fuji: {
      url: 'https://rpc.ankr.com/avalanche_fuji',
      accounts: [SECRET_KEY]
    },
    bsctestnet: {
      url: 'https://data-seed-prebsc-1-s3.binance.org:8545',
      accounts: [SECRET_KEY]
    },
    lineagoerli: {
      url: 'https://linea-goerli.infura.io/v3/47f445c73c594312b339de8afbecc99d',
      accounts: [SECRET_KEY]
    },
    arbitrumgoerli: {
      url: 'https://goerli-rollup.arbitrum.io/rpc',
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
