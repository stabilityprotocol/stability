interface Ownable2Step {
    event NewOwner(address newOwner);

    /// @dev Returns owner.
    /// @custom:selector 8da5cb5b
    function owner() external view returns (address);

    /// @dev Returns pending claimable owner.
    /// @custom:selector e30c3978
    function pendingOwner() external view returns (address);

    /// @dev Returns pending claimable owner.
    /// @custom:selector f2fde38b
    function transferOwnership(address newOwner) external; // onlyOwner

    /// @dev Returns pending claimable owner.
    /// @custom:selector 79ba5097
    function acceptOwnership() external; // only pendingOwner()
}
