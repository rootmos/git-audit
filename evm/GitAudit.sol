pragma solidity ^0.5.11;

contract GitAudit {
    uint160[] _commits = new uint160[](0);
    address public owner = msg.sender;

    modifier onlyBy(address _account) {
        require(msg.sender == _account);
        _;
    }

    function anchor(uint160 commit) onlyBy(owner) public {
        _commits.push(commit);
    }

    function commits() public view returns (uint160[] memory) {
        return _commits;
    }
}
