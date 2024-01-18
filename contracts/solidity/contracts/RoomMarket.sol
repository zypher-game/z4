// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";

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

contract RoomMarket is OwnableUpgradeable {
    struct Room {
        mapping(address => bool) exists;
        address[] players;
        bytes32[] pubkeys;
        address creator;
        uint256 reward;
        address sequencer;
        uint256 locked;
        uint256 site;
        RoomStatus status;
    }

    struct Sequencer {
        string http;
        uint256 staking;
    }

    /// main token address
    address public token;

    /// min staking before accept room
    uint256 public minStaking;

    /// every player every room lock staking
    uint256 public playerRoomLock;

    /// next room id, start from 100000
    uint256 public nextRoomId = 100000;

    /// waiting & running rooms
    mapping(uint256 => Room) public rooms;

    /// registered sequencers
    mapping(address => Sequencer) public sequencers;

    event StakeSequencer(address sequencer, string http, uint256 staking);
    event UnstakeSequencer(address sequencer, uint256 staking);
    event StartRoom(uint256 room, uint256 reward, address[] players, bytes32[] pubkeys);
    event AcceptRoom(uint256 room, address sequencer, uint256 locked);
    event OverRoom(uint256 room);

    function initialize(address _token, uint256 _minStaking, uint256 _playerRoomLock) external initializer {
        token = _token;
        minStaking = _minStaking;
        playerRoomLock = _playerRoomLock;

        // init
        __Ownable_init(msg.sender);
    }

    function setToken(address _token) external onlyOwner {
        token = _token;
    }

    function setMinStaking(uint256 _minStaking) external onlyOwner {
        minStaking = _minStaking;
    }

    function setPlayerRoomLock(uint256 _playerRoomLock) external onlyOwner {
        playerRoomLock = _playerRoomLock;
    }

    function isSequencer(address sequencer) external view returns (bool) {
        return sequencers[sequencer].staking >= minStaking;
    }

    function stakeSequencer(string calldata http, uint256 amount) external {
        IERC20(token).transferFrom(msg.sender, address(this), amount);

        Sequencer storage sequencer = sequencers[msg.sender];
        sequencer.staking += amount;
        sequencer.http = http;

        emit StakeSequencer(msg.sender, http, sequencer.staking);
    }

    function unstakeSequencer(uint256 amount) external {
        Sequencer storage sequencer = sequencers[msg.sender];
        require(sequencer.staking >= amount, "RM01");

        sequencer.staking -= amount;
        IERC20(token).transfer(msg.sender, amount);

        emit UnstakeSequencer(msg.sender, sequencer.staking);
    }

    function createRoom(uint256 reward, uint256 limit, address player, bytes32 pubkey) external returns (uint256) {
        nextRoomId += 1;

        Room storage room = rooms[nextRoomId];
        room.exists[player] = true;
        room.players.push(player);
        room.pubkeys.push(pubkey);
        room.creator = msg.sender;
        room.reward = reward;
        room.site = limit - 1;
        room.status = RoomStatus.Opening;

        return nextRoomId;
    }

    function joinRoom(uint256 roomId, address player, bytes32 pubkey) external returns (uint256) {
        Room storage room = rooms[roomId];
        require(room.status == RoomStatus.Opening, "RM02");
        require(room.site > 0 && !room.exists[player], "RM03");

        room.exists[player] = true;
        room.players.push(player);
        room.pubkeys.push(pubkey);
        room.site -= 1;

        if (room.site == 0) {
            room.status = RoomStatus.Waiting;
            emit StartRoom(roomId, room.reward, room.players, room.pubkeys);
        }

        return room.site;
    }

    function startRoom(uint256 roomId) external {
        Room storage room = rooms[roomId];
        require(room.creator == msg.sender, "RM04");
        require(room.status == RoomStatus.Opening, "RM02");

        room.status = RoomStatus.Waiting;
        emit StartRoom(roomId, room.reward, room.players, room.pubkeys);
    }


    function acceptRoom(uint256 roomId) external {
        Sequencer storage sequencer = sequencers[msg.sender];
        Room storage room = rooms[roomId];

        uint256 lockAmount = room.players.length * playerRoomLock;
        require(sequencer.staking > minStaking && sequencer.staking > lockAmount, "RM05");
        require(room.status == RoomStatus.Waiting, "RM02");

        room.sequencer = msg.sender;
        room.status = RoomStatus.Playing;
        room.locked = lockAmount;

        sequencer.staking -= lockAmount;

        emit AcceptRoom(roomId, msg.sender, lockAmount);
    }

    function overRoomWithZk(uint256 roomId, bytes calldata data, bytes calldata proof) external {
        Room storage room = rooms[roomId];
        require(room.status == RoomStatus.Playing, "RM02");
        require(room.sequencer == msg.sender, "RM06");
        require(proof.length > 0 && data.length > 0, "RM07");

        // TODO callback & verify zkp

        _overRoom(roomId);
    }

    function overRoomWithSign(uint256 roomId, bytes calldata data, bytes[] calldata proofs) external {
        Room storage room = rooms[roomId];
        require(room.status == RoomStatus.Playing, "RM02");
        require(room.sequencer == msg.sender, "RM06");
        require(proofs.length <= room.players.length && proofs.length >= room.players.length / 2 && data.length > 0, "RM07");

        // TODO callback & verify sign

        _overRoom(roomId);

    }

    function _overRoom(uint256 roomId) private {
        Room storage room = rooms[roomId];
        Sequencer storage sequencer = sequencers[room.sequencer];

        sequencer.staking += room.locked;
        sequencer.staking += room.reward;

        delete rooms[roomId];
        emit OverRoom(roomId);
    }

    // TODO conflict: if room is not playing at sequencer, creator can restart it.
    function restartRoom(uint256 roomId) external {
        Room storage room = rooms[roomId];
        require(room.status == RoomStatus.Playing, "RM02");
        require(room.creator == msg.sender, "RM04");

        room.status = RoomStatus.Waiting;
        emit StartRoom(roomId, room.reward, room.players, room.pubkeys);
    }
}
