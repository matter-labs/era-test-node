// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract EventRegistrationNFT {
    // Mapping from token ID to owner address
    mapping(uint256 => address) private _owners;

    // Mapping from owner address to number of owned tokens
    mapping(address => uint256) private _balances;

    // Mapping from token ID to approved address
    mapping(uint256 => address) private _tokenApprovals;

    // Mapping from owner to operator approvals
    mapping(address => mapping(address => bool)) private _operatorApprovals;

    // Total supply of NFTs
    uint256 private _totalSupply;

    // Token name
    string private _name;

    // Token symbol
    string private _symbol;

    // Minting price (1 ETH)
    uint256 public constant MINT_PRICE = 1 ether;

    // Event emitted when an NFT is minted
    event Transfer(
        address indexed from,
        address indexed to,
        uint256 indexed tokenId
    );

    event Approval(
        address indexed owner,
        address indexed approved,
        uint256 indexed tokenId
    );

    event ApprovalForAll(
        address indexed owner,
        address indexed operator,
        bool approved
    );

    constructor() {
        _name = "EventTicket";
        _symbol = "ETICKET";
    }

    // Function to get token name
    function name() external view returns (string memory) {
        return _name;
    }

    // Function to get token symbol
    function symbol() external view returns (string memory) {
        return _symbol;
    }

    // Function to get the balance of tokens held by an address
    function balanceOf(address owner) public view returns (uint256) {
        require(owner != address(0), "Invalid address");
        return _balances[owner];
    }

    // Function to get the owner of a token by ID
    function ownerOf(uint256 tokenId) public view returns (address) {
        address owner = _owners[tokenId];
        require(owner != address(0), "Token does not exist");
        return owner;
    }

    // Internal function to mint a new token
    function _mint(address to, uint256 tokenId) internal {
        require(to != address(0), "Invalid address");
        require(_owners[tokenId] == address(0), "Token already minted");

        _balances[to] += 1;
        _owners[tokenId] = to;

        emit Transfer(address(0), to, tokenId);
    }

    // Function to register for an event (mint NFT)
    function registerForEvent() external payable returns (uint256) {
        require(msg.value == MINT_PRICE, "Minting a ticket costs 1 ETH");

        _totalSupply += 1;
        uint256 newTokenId = _totalSupply;

        _mint(msg.sender, newTokenId);

        return newTokenId;
    }

    // Function to transfer the token
    function transferFrom(address from, address to, uint256 tokenId) public {
        require(
            _isApprovedOrOwner(msg.sender, tokenId),
            "Not approved or owner"
        );
        _transfer(from, to, tokenId);
    }

    // Internal function to transfer ownership of a token
    function _transfer(address from, address to, uint256 tokenId) internal {
        require(ownerOf(tokenId) == from, "Not the owner");
        require(to != address(0), "Invalid recipient address");

        // Clear approvals
        _approve(address(0), tokenId);

        _balances[from] -= 1;
        _balances[to] += 1;
        _owners[tokenId] = to;

        emit Transfer(from, to, tokenId);
    }

    // Function to approve another address to transfer a token
    function approve(address to, uint256 tokenId) public {
        address owner = ownerOf(tokenId);
        require(to != owner, "Cannot approve the owner");
        require(
            msg.sender == owner || isApprovedForAll(owner, msg.sender),
            "Not approved"
        );

        _approve(to, tokenId);
    }

    // Internal function to approve an address for a token
    function _approve(address to, uint256 tokenId) internal {
        _tokenApprovals[tokenId] = to;
        emit Approval(ownerOf(tokenId), to, tokenId);
    }

    // Function to check if an address is approved or the owner of a token
    function _isApprovedOrOwner(
        address spender,
        uint256 tokenId
    ) internal view returns (bool) {
        address owner = ownerOf(tokenId);
        return (spender == owner ||
            getApproved(tokenId) == spender ||
            isApprovedForAll(owner, spender));
    }

    // Function to get the approved address for a token
    function getApproved(uint256 tokenId) public view returns (address) {
        require(_owners[tokenId] != address(0), "Token does not exist");
        return _tokenApprovals[tokenId];
    }

    // Function to set approval for all tokens of a certain owner
    function setApprovalForAll(address operator, bool approved) public {
        require(operator != msg.sender, "Cannot approve yourself");
        _operatorApprovals[msg.sender][operator] = approved;
        emit ApprovalForAll(msg.sender, operator, approved);
    }

    // Function to check if an operator is approved for all tokens of an owner
    function isApprovedForAll(
        address owner,
        address operator
    ) public view returns (bool) {
        return _operatorApprovals[owner][operator];
    }

    // Function to withdraw the accumulated funds by the owner
    function withdraw() external {
        uint256 balance = address(this).balance;
        require(balance > 0, "No funds to withdraw");
        payable(msg.sender).call{value: balance};
    }
}
