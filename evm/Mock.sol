pragma solidity ^0.5.11;

contract Mock {
    function echo(uint256 i) public pure returns (uint256) {
        return i;
    }

    bool private fail = true;

    function maybe_fail(string memory s) public view returns (string memory) {
        if(!fail) {
            return s;
        } else {
            revert(s);
        }
    }

    function set_fail(bool f) public {
        fail = f;
    }
}
