// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

enum RoomStatus {
    // the room is over or not exist
    Over,
    // room is opening for all players
    Opening,
    // waiting sequencer accept the offer
    Waiting,
    // room is playing
    Playing
}

contract RoomMarket {
    struct Room {
        mapping(address => bool) exists;
        address[] players;
        bytes32[] pubkeys;
        address creator;
        address sequencer;
        uint256 site;
        RoomStatus status;
    }

    struct Sequencer {
        string http;
        uint256 staking;
    }

    uint256 public nextRoomId = 100000; // start from 100000

    mapping(uint256 => Room) public rooms;

    mapping(address => Sequencer) public sequencers;

    event RegisterSequencer(address sequencer, string http, uint256 staking);
    event StartRoom(uint256 room, address[] players, bytes32[] pubkeys);
    event AcceptRoom(uint256 room, address sequencer);
    event OverRoom(uint256 room);

    function registerSequencer(string calldata http, uint256 staking) public {
        require(sequencers[msg.sender].staking == 0, "HAD SEQUENCER");

        Sequencer storage sequencer = sequencers[msg.sender];
        sequencer.staking = staking;
        sequencer.http = http;

        emit RegisterSequencer(msg.sender, http, staking);
    }

    function createRoom(uint256 limit, address player, bytes32 pubkey) public returns (uint256) {
        nextRoomId += 1;

        Room storage room = rooms[nextRoomId];
        room.exists[player] = true;
        room.players.push(player);
        room.pubkeys.push(pubkey);
        room.creator = msg.sender;
        room.site = limit - 1;
        room.status = RoomStatus.Opening;

        return nextRoomId;
    }

    function joinRoom(uint256 roomId, address player, bytes32 pubkey) public returns (uint256) {
        Room storage room = rooms[roomId];
        require(room.status == RoomStatus.Opening, "NOT OPENING");
        require(room.site > 0 && !room.exists[player], "FULL");

        room.exists[player] = true;
        room.players.push(player);
        room.pubkeys.push(pubkey);
        room.site -= 1;

        if (room.site == 0) {
            room.status = RoomStatus.Waiting;
            emit StartRoom(roomId, room.players, room.pubkeys);
        }

        return room.site;
    }

    function startRoom(uint256 roomId) public {
        Room storage room = rooms[roomId];
        require(room.creator == msg.sender, "PERMISSION");
        require(room.status == RoomStatus.Opening, "NOT OPENING");

        room.status = RoomStatus.Waiting;
        emit StartRoom(roomId, room.players, room.pubkeys);
    }

    function acceptRoom(uint256 roomId) public {
        require(sequencers[msg.sender].staking > 0, "NOT SEQUENCER");

        Room storage room = rooms[roomId];
        require(room.status == RoomStatus.Waiting, "NOT WAITING");

        room.sequencer = msg.sender;
        room.status = RoomStatus.Playing;

        emit AcceptRoom(roomId, msg.sender);
    }

    // TODO: if room is not playing at sequencer, creator can restart it.
    function restartRoom(uint256 roomId) public {
        Room storage room = rooms[roomId];
        require(room.status == RoomStatus.Playing, "NOT PLAYING");
        require(room.creator == msg.sender, "PERMISSION");

        room.status = RoomStatus.Waiting;
        emit StartRoom(roomId, room.players, room.pubkeys);
    }

    function overRoom(uint256 roomId, bytes calldata data, bytes calldata proof) public {
        Room storage room = rooms[roomId];
        require(room.status == RoomStatus.Playing, "NOT PLAYING");
        require(room.sequencer == msg.sender, "NOT SEQUENCER");
        require(proof.length > 0 && data.length > 0, "INVALID PROOF");

        // TODO callback & verify zkp

        delete rooms[roomId];
        emit OverRoom(roomId);
    }
}
