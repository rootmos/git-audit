pragma solidity ^0.5.11;

contract GitAudit {
    uint160[] _commits = new uint160[](0);

    function anchor(uint160 commit) public {
        _commits.push(commit);
    }

    function commits() public view returns (uint160[] memory) {
        return _commits;
    }
}
