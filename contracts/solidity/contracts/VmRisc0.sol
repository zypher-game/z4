// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";

import './RoomMarket.sol';

contract VmRisc0 is OwnableUpgradeable {
    struct Room {
        uint256 game;
        address[] players;
    }

    struct Game {
        address owner;
        uint256 playerLimit;
        uint256 resultLimit;
        uint256 version;
        bool actived;
    }

    /// z4 common room market
    address roomMarket;

    /// next uid for game
    uint256 nextGame = 0;

    /// waiting & running rooms
    mapping(uint256 => Room) private rooms;

    mapping(uint256 => Game) private games;

    event AddGame(uint256 game, address owner, uint256 playerLimit, uint256 resultLimit, uint256 version, bytes program);
    event ActiveGame(uint256 game, bool actived);
    event DelGame(uint256 game);

    constructor(address _roomMarket) {
        roomMarket = _roomMarket;

        __Ownable_init(msg.sender);
    }

    function setRoomMarket(address _roomMarket) external onlyOwner {
        roomMarket = _roomMarket;
    }

    function addGame(uint256 playerLimit, uint256 resultLimit, bytes calldata program) external returns (uint256) {
        games[nextGame] = Game(msg.sender, playerLimit, resultLimit, 1, true);
        emit AddGame(nextGame, msg.sender, playerLimit, resultLimit, 1, program);
        nextGame += 1;

        return nextGame - 1;
    }

    function updateGame(uint256 gameId, uint256 playerLimit, uint256 resultLimit, bytes calldata program) external {
        require(games[gameId].owner == msg.sender, "VR01");

        Game storage game = games[gameId];
        game.playerLimit = playerLimit;
        game.resultLimit = resultLimit;
        game.version += 1;

        emit AddGame(gameId, game.owner, playerLimit, resultLimit, game.version, program);
    }

    function delGame(uint256 gameId) external {
        require(games[gameId].owner == msg.sender, "VR01");
        delete games[gameId];

        emit DelGame(gameId);
    }

    function activeGame(uint256 gameId, bool actived) external onlyOwner {
        games[gameId].actived = actived;

        emit ActiveGame(gameId, actived);
    }

    function createRoom(uint256 gameId, address peer) external {
        // TODO reward
        uint256 rewards = 0;
        Game storage game = games[gameId];
        uint256 roomId = RoomMarket(roomMarket).createRoom(rewards, game.playerLimit, msg.sender, peer);
        rooms[roomId].players.push(msg.sender);
    }

    function joinRoom(uint256 roomId, address peer) external {
        RoomMarket(roomMarket).joinRoom(roomId, msg.sender, peer);
        rooms[roomId].players.push(msg.sender);
    }

    function startRoom(uint256 roomId) external {
        require(rooms[roomId].players[0] == msg.sender, "D01");
        RoomMarket(roomMarket).startRoom(roomId);
    }

    function claimRoom(uint256 roomId) external {
        //
    }
}
