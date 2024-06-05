// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

enum RoomStatus {
    // the room not exist
    None,
    // room is opening for all players
    Opening,
    // waiting sequencer accept the offer
    Waiting,
    // room is playing
    Playing,
    // the room is over
    Over
}

abstract contract RoomMarket is Ownable {
    struct Room {
        mapping(address => bool) exists;
        address[] players;
        address[] peers;
        bytes32[] pks;
        bool viewable;
        uint256 ticket;
        uint256 reward;
        bytes32 salt;
        bytes32 block;
        address sequencer;
        uint256 locked;
        uint256 site;
        bytes   result;
        RoomStatus status;
    }

    struct Sequencer {
        string http;
        string websocket;
        uint256 staking;
    }

    /// main token address
    address public token;

    /// min staking before accept room
    uint256 public minStaking;

    /// every player every room lock staking
    uint256 public playerRoomLock;

    /// next room id, start from 100000
    uint256 public nextRoomId;

    /// player number in every room
    uint256 public playerLimit;

    /// waiting & running rooms
    mapping(uint256 => Room) public rooms;

    /// registered sequencers
    mapping(address => Sequencer) public sequencers;

    event StakeSequencer(address sequencer, string http, string websocket, uint256 staking);
    event UnstakeSequencer(address sequencer, uint256 staking);
    event CreateRoom(uint256 room, address game, uint256 reward, bool viewable, address player, address peer, bytes32 pk, bytes32 salt, bytes32 block);
    event JoinRoom(uint256 room, address player, address peer, bytes32 pk);
    event StartRoom(uint256 room, address game);
    event AcceptRoom(uint256 room, address sequencer, string websocket, uint256 locked, bytes params);
    event OverRoom(uint256 room);
    event ClaimRoom(uint256 room);

    constructor(address _token, uint256 _minStaking, uint256 _playerRoomLock, uint256 _playerLimit, uint256 _startRoomId) Ownable(msg.sender) {
        token = _token;
        minStaking = _minStaking;
        playerRoomLock = _playerRoomLock;
        playerLimit = _playerLimit;
        nextRoomId = _startRoomId;
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

    function setPlayerLimit(uint256 _playerLimit) external onlyOwner {
        playerLimit = _playerLimit;
    }

    function isSequencer(address sequencer) external view returns (bool) {
        return sequencers[sequencer].staking >= minStaking;
    }

    function stakeSequencer(string calldata http, string calldata websocket, uint256 amount) external {
        IERC20(token).transferFrom(msg.sender, address(this), amount);

        Sequencer storage sequencer = sequencers[msg.sender];
        sequencer.staking += amount;
        sequencer.http = http;
        sequencer.websocket = websocket;

        emit StakeSequencer(msg.sender, http, websocket, sequencer.staking);
    }

    function unstakeSequencer(uint256 amount) external {
        Sequencer storage sequencer = sequencers[msg.sender];
        require(sequencer.staking >= amount, "RM01");

        sequencer.staking -= amount;
        IERC20(token).transfer(msg.sender, amount);

        emit UnstakeSequencer(msg.sender, sequencer.staking);
    }

    function createRoom(uint256 ticket, bool viewable, address peer, bytes32 pk, bytes32 salt) external returns (uint256) {
        // TODO Transfer ticket to contract

        Room storage room = rooms[nextRoomId];
        room.exists[msg.sender] = true;
        room.players.push(msg.sender);
        room.peers.push(peer);
        room.pks.push(pk);

        room.viewable = viewable;
        room.ticket = ticket;
        room.reward = ticket;
        room.salt = salt;
        room.block = bytes32(block.prevrandao); // merge prevrandao and salt as the room random seed
        room.site = playerLimit - 1;
        room.status = RoomStatus.Opening;

        nextRoomId += 1;
        emit CreateRoom(nextRoomId - 1, address(this), room.reward, viewable, msg.sender, peer, pk, salt, room.block);

        return nextRoomId - 1;
    }

    function joinRoom(uint256 roomId, address peer, bytes32 pk) external returns (uint256) {
        Room storage room = rooms[roomId];
        require(room.status == RoomStatus.Opening, "RM02");
        require(room.site > 0 && !room.exists[msg.sender], "RM03");

        // TODO Transfer ticket to contract

        room.exists[msg.sender] = true;
        room.players.push(msg.sender);
        room.peers.push(peer);
        room.pks.push(pk);

        room.reward += room.ticket;
        room.site -= 1;

        emit JoinRoom(roomId, msg.sender, peer, pk);

        if (room.site == 0) {
            room.status = RoomStatus.Waiting;
            emit StartRoom(roomId, address(this));
        }

        return room.site;
    }

    function startRoom(uint256 roomId) external {
        Room storage room = rooms[roomId];
        require(room.status == RoomStatus.Opening, "RM02");

        room.status = RoomStatus.Waiting;
        emit StartRoom(roomId, address(this));
    }

    function acceptRoom(uint256 roomId, bytes calldata params) external {
        Room storage room = rooms[roomId];
        Sequencer storage sequencer = sequencers[msg.sender];

        uint256 lockAmount = room.players.length * playerRoomLock;
        require(sequencer.staking >= minStaking && sequencer.staking >= lockAmount, "RM04");
        require(room.status == RoomStatus.Waiting, "RM02");

        room.sequencer = msg.sender;
        room.status = RoomStatus.Playing;
        room.locked = lockAmount;

        sequencer.staking -= lockAmount;

        emit AcceptRoom(roomId, msg.sender, sequencer.websocket, lockAmount, params);
    }

    function overRoomWithZk(uint256 roomId, bytes calldata data, bytes calldata proof) external {
        Room storage room = rooms[roomId];
        require(room.status == RoomStatus.Playing, "RM02");
        require(room.sequencer == msg.sender, "RM05");

        // TODO require(proof.length > 0 && data.length > 0, "RM07");
        // TODO callback & verify zkp
        bytes32 seed = room.salt ^ room.block;

        room.result = data;
        _overRoom(roomId);
    }

    function overRoomWithThreshold(uint256 roomId, bytes calldata data, bytes calldata proof) external {
        Room storage room = rooms[roomId];
        require(room.status == RoomStatus.Playing, "RM02");
        require(room.sequencer == msg.sender, "RM05");

        // verify sign
        bytes32 hash = keccak256(abi.encodePacked('\x19Ethereum Signed Message:\n32', roomId, data));
        // address signer = ECDSA.recover(hash, proof);

        room.result = data;
        _overRoom(roomId);

    }

    function _overRoom(uint256 roomId) private {
        Room storage room = rooms[roomId];
        Sequencer storage sequencer = sequencers[room.sequencer];

        sequencer.staking += room.locked;
        sequencer.staking += room.reward;

        room.status = RoomStatus.Over;

        claimRoom(roomId);
        emit OverRoom(roomId);
    }

    // suggest rewrite the claimRoom
    function claimRoom(uint256 roomId) public virtual {
        delete rooms[roomId];
        emit ClaimRoom(roomId);
    }

    // TODO conflict: if room is not playing at sequencer, creator can restart it.
    function restartRoom(uint256 roomId) external {
        Room storage room = rooms[roomId];
        require(room.status == RoomStatus.Playing, "RM02");
        require(room.players[0] == msg.sender, "RM06");

        room.status = RoomStatus.Waiting;
        emit StartRoom(roomId, address(this));
    }

    function roomInfo(uint256 roomId) external view returns (address[] memory, address, address, uint256, RoomStatus) {
        Room storage room = rooms[roomId];
        return (room.players, address(this), room.sequencer, room.site, room.status);
    }
}
