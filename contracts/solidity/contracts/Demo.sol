// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";

import './RoomMarket.sol';

contract Demo is OwnableUpgradeable {
    struct Room {
        address[] players;
    }

    /// z4 common room market
    address roomMarket;

    uint256 playerLimit = 4;

    /// waiting & running rooms
    mapping(uint256 => Room) private rooms;

    function initialize(address _roomMarket) external initializer {
        // init
        roomMarket = _roomMarket;
        __Ownable_init(msg.sender);
    }

    function setRoomMarket(address _roomMarket) external onlyOwner {
        roomMarket = _roomMarket;
    }

    function setPlayerLimit(uint256 _playerLimit) external onlyOwner {
        playerLimit = _playerLimit;
    }

    function createRoom(bytes32 pubkey) external {
        // TODO reward
        uint256 roomId = RoomMarket(roomMarket).createRoom(0, playerLimit, msg.sender, pubkey);
        rooms[roomId].players.push(msg.sender);
    }

    function joinRoom(uint256 roomId, bytes32 pubkey) external {
        RoomMarket(roomMarket).joinRoom(roomId, msg.sender, pubkey);
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