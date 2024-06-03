// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

import "./RoomMarket.sol";

contract SimpleGame is RoomMarket {
    struct Rank {
        uint256 win;
        uint256 reward;
    }

    mapping(address => Rank) ranking;

    event Ranking(address, uint256 win, uint256 reward);

    constructor(address _token, uint256 _minStaking, uint256 _playerRoomLock, uint256 _playerLimit, uint256 _startRoomId) RoomMarket(
        _token, _minStaking, _playerRoomLock, _playerLimit, _startRoomId
    ) {}

    function claimRoom(uint256 roomId) public override {
        Room storage room = rooms[roomId];
        require(room.status == RoomStatus.Over, "SG01");

        (address[] memory winners) = abi.decode(room.result, (address[]));
        if (winners.length > 0) {
            uint256 amount = room.reward - room.ticket;

            // TODO transfer limit-1/limit reward to winner

            ranking[winners[0]].win += 3;
            ranking[winners[0]].reward += amount;
            emit Ranking(winners[0], 3, amount);
        }

        if (winners.length > 1) {
            // TODO transfer ticket reward to winner
            uint256 amount = room.ticket;

            ranking[winners[1]].win += 1;
            ranking[winners[1]].reward += amount;
            emit Ranking(winners[1], 1, amount);
        }

        delete rooms[roomId];
        emit ClaimRoom(roomId);
    }
}
