object "Console" {
	
	code { }
	object "Console_deployed" {
		code {
			let log_address := 0x000000000000000000636F6e736F6c652e6c6f67
            // load the free memory pointer
            let freeMemPointer := mload(0x40)
            // store the function selector of log(uint256) in memory
            mstore(freeMemPointer, 0xf82c50f1)
            // store the first argument of log(uint256) in the next memory slot
            mstore(add(freeMemPointer, 0x20), 3)
            // update the free memory pointer
            mstore(0x40, add(freeMemPointer, 0x40))
            // memory will look like:
            //  00000000000000000000000000000000000000000000000000000000f82c50f1
            //  0000000000000000000000000000000000000000000000000000000000000003
            // call the sum function of contract A
            // and store the result in memory slot 0x00
            if iszero(staticcall(gas(), log_address, add(freeMemPointer, 28), mload(0x40), 0x00, 0x00)) {
                revert(0,0)
            }
            // return the result from memory slot 0x00
            return(0x40, 0x40)
		}
	}
}
