// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";

import './RoomMarket.sol';

contract Demo is Ownable {
    struct Room {
        address[] players;
    }

    /// z4 common room market
    address roomMarket;

    uint256 playerLimit = 4;

    /// waiting & running rooms
    mapping(uint256 => Room) private rooms;

    constructor(address _roomMarket) Ownable(msg.sender) {
        roomMarket = _roomMarket;
    }

    function setRoomMarket(address _roomMarket) external onlyOwner {
        roomMarket = _roomMarket;
    }

    function setPlayerLimit(uint256 _playerLimit) external onlyOwner {
        playerLimit = _playerLimit;
    }

    function createRoom(address peer, bytes32 pk) external {
        // TODO reward
        uint256 roomId = RoomMarket(roomMarket).createRoom(0, 0, playerLimit, msg.sender, peer, pk);
        rooms[roomId].players.push(msg.sender);
    }

    function joinRoom(uint256 roomId, address peer, bytes32 pk) external {
        RoomMarket(roomMarket).joinRoom(roomId, msg.sender, peer, pk);
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
