// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

contract RoomMarket {
    struct Room {
        address[] players;
        address sequencer;
        bool over;
    }

    struct Sequencer {
        string http;
        uint256 staking;
    }

    uint256 public nextRoomId;

    mapping(uint256 => Room) public rooms;

    mapping(address => Sequencer) public sequencers;

    event RegisterSequencer(address sequencer, string http, uint256 staking);
    event CreateRoom(uint256 roomId, address player);
    event JoinRoom(uint256 roomId, address player);
    event StartRoom(uint256 roomId, address sequencer);
    event OverRoom(uint256 roomId);

    function registerSequencer(string calldata http, uint256 staking) public {
        require(sequencers[msg.sender].staking == 0, "HAD SEQUENCER");

        Sequencer storage sequencer = sequencers[msg.sender];
        sequencer.staking = staking;
        sequencer.http = http;

        emit RegisterSequencer(msg.sender, http, staking);
    }

    function createRoom(address player) public returns (uint256) {
        nextRoomId += 1;

        Room storage room = rooms[nextRoomId];
        room.players.push(player);

        emit CreateRoom(nextRoomId, player);

        return nextRoomId;
    }

    function joinRoom(uint256 roomId, address player) public {
        Room storage room = rooms[roomId];
        require(room.players.length < 4, "FULL");

        room.players.push(player);
        emit CreateRoom(roomId, player);
    }

    function startRoom(uint256 roomId) public {
        require(sequencers[msg.sender].staking > 0, "NOT SEQUENCER");

        Room storage room = rooms[roomId];
        require(room.players.length == 4, "WAITING");

        room.sequencer = msg.sender;
        emit StartRoom(roomId, msg.sender);
    }

    function overRoom(uint256 roomId, bytes calldata data, bytes calldata proof) public {
        Room storage room = rooms[roomId];

        require(proof.length > 0 && data.length > 0, "INVALID PROOF");
        require(room.sequencer == msg.sender, "NOT SEQUENCER");
        require(room.players.length == 4 && !room.over, "NO ROOM");

        // TODO callback & verify zkp

        emit OverRoom(roomId);
    }
}
