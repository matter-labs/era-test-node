object "ModExp" {
	code { }
	object "ModExp_deployed" {
		code {
            ////////////////////////////////////////////////////////////////
            //                      CONSTANTS
            ////////////////////////////////////////////////////////////////

            function ZERO() -> zero {
                zero := 0x0
            }

            function ONE() -> one {
                one := 0x1
            }

            function TWO() -> two {
                two := 0x2
            }

            function WORD_SIZE() -> word {
                word := 0x20
            }

            //////////////////////////////////////////////////////////////////
            //                      HELPER FUNCTIONS
            //////////////////////////////////////////////////////////////////

            // @dev Packs precompile parameters into one word.
            // Note: functions expect to work with 32/64 bits unsigned integers.
            // Caller should ensure the type matching before!

            /// @dev Executes the `precompileCall` opcode.
            function precompileCall(precompileParams, gasToBurn) -> ret {
                // Compiler simulation for calling `precompileCall` opcode
                ret := verbatim_2i_1o("precompile", precompileParams, gasToBurn)
            }

            function exponentIsZero(exponent_limbs, exponent_pointer) -> isZero {
                isZero := 0x00
                let next_limb_pointer := exponent_pointer
                for { let limb_number := 0 } lt(limb_number, exponent_limbs) { limb_number := add(limb_number, ONE()) } {
                    let limb := mload(next_limb_pointer)
                    isZero := or(isZero, limb)
                    if isZero {
                        break
                    }
                    next_limb_pointer := add(next_limb_pointer, WORD_SIZE())
                }
                isZero := iszero(isZero)
            }

            // CONSOLE.LOG Caller
            // It prints 'val' in the node console and it works using the 'mem'+0x40 memory sector
            function console_log(mem, val) -> {
                let log_address := 0x000000000000000000636F6e736F6c652e6c6f67
                // load the free memory pointer
                let freeMemPointer := mload(mem)
                // store the function selector of log(uint256) in memory
                mstore(freeMemPointer, 0xf82c50f1)
                // store the first argument of log(uint256) in the next memory slot
                mstore(add(freeMemPointer, 0x20), val)
                // call the console.log contract
                if iszero(staticcall(gas(),log_address,add(freeMemPointer, 28),add(freeMemPointer, 0x40),0x00,0x00)) {
                    revert(0,0)
                }
            }

            ////////////////////////////////////////////////////////////////
            //                      FALLBACK
            ////////////////////////////////////////////////////////////////

            let base_length := calldataload(0)
            let exponent_length := calldataload(32)
            let modulus_length := calldataload(64)

            if iszero(gt(calldatasize(), 96)) {
                return(0, 0)
            }

            // Handle a special case when both the base and mod length is zero
            if and(iszero(base_length), iszero(modulus_length)) {
                return(0, 0)
            }

            let base_pointer := 96
            let base_padding := sub(WORD_SIZE(), base_length)
            let padded_base_pointer := add(96, base_padding)
            calldatacopy(padded_base_pointer, base_pointer, base_length)
            let base := mload(base_pointer)
            
            let calldata_exponent_pointer := add(base_pointer, base_length)
            let memory_exponent_pointer := add(base_pointer, WORD_SIZE())
            let exponent_limbs := ZERO()
            switch iszero(mod(exponent_length, WORD_SIZE()))
            case 0 {
                exponent_limbs := add(div(exponent_length, WORD_SIZE()), ONE())
            }
            case 1 {
                exponent_limbs := div(exponent_length, WORD_SIZE())
            }
            // The exponent expected length given the amount of limbs.
            let adjusted_exponent_length := mul(WORD_SIZE(), exponent_limbs)
            let calldata_next_limb_pointer := calldata_exponent_pointer
            let memory_next_limb_pointer := memory_exponent_pointer
            for { let limb_number := 0 } lt(limb_number, exponent_limbs) { limb_number := add(limb_number, ONE()) } {
                // The msb of the leftmost limb could be one.
                // This left-pads with zeros the leftmost limbs to achieve 32 bytes.
                if iszero(limb_number) {
                    // The amount of zeros to left-pad.
                    let padding := sub(adjusted_exponent_length, exponent_length)
                    // This is either 0 or > 0 if there are any zeros to pad.
                    let padded_exponent_pointer := add(memory_exponent_pointer, padding)
                    let amount_of_bytes_for_first_limb := sub(WORD_SIZE(), padding)
                    calldatacopy(padded_exponent_pointer, calldata_exponent_pointer, amount_of_bytes_for_first_limb)
                    calldata_next_limb_pointer := add(calldata_exponent_pointer, amount_of_bytes_for_first_limb)
                    memory_next_limb_pointer := add(memory_exponent_pointer, WORD_SIZE())
                    continue
                }
                calldatacopy(memory_next_limb_pointer, calldata_next_limb_pointer, WORD_SIZE())
                calldata_next_limb_pointer := add(calldata_next_limb_pointer, WORD_SIZE())
                memory_next_limb_pointer := add(memory_next_limb_pointer, WORD_SIZE())
            }

            let calldata_modulus_pointer := add(calldata_exponent_pointer, exponent_length)
            let memory_modulus_pointer := add(memory_exponent_pointer, adjusted_exponent_length)
            calldatacopy(add(memory_modulus_pointer, sub(WORD_SIZE(), modulus_length)), calldata_modulus_pointer, modulus_length)

            let modulus := mload(memory_modulus_pointer)

            // 1^exponent % modulus = 1
            if eq(base, ONE()) {
                mstore(0, ONE())
                let unpadding := sub(WORD_SIZE(), modulus_length)
                return(unpadding, modulus_length)
            }

            // base^exponent % 0 = 0
            if iszero(modulus) {
                mstore(0, ZERO())
                return(0, modulus_length)
            }

            // base^0 % modulus = 1
            if exponentIsZero(exponent_length, memory_exponent_pointer) {
                console_log(0x600, 0xf)
                mstore(0, ONE())
                let unpadding := sub(WORD_SIZE(), modulus_length)
                return(unpadding, modulus_length)
            }

            // 0^exponent % modulus = 0
            console_log(0x600, base)
            if iszero(base) {
                console_log(0x600, 0xf)
                mstore(0, ZERO())
                return(0, modulus_length)
            }

            switch eq(exponent_limbs, ONE())
            case 1 {
                let pow := 1
                // If we have one limb, then the exponent has 32 bytes and it is
                // located in 0x
                let exponent := mload(memory_exponent_pointer)
                base := mod(base, modulus)
                for { let i := 0 } gt(exponent, ZERO()) { i := add(i, 1) } {
                    if eq(mod(exponent, TWO()), ONE()) {
                        pow := mulmod(pow, base, modulus)
                    }
                    exponent := shr(1, exponent)
                    base := mulmod(base, base, modulus)
                }
    
                mstore(0, pow)
                let unpadding := sub(WORD_SIZE(), modulus_length)
                return(unpadding, modulus_length)
            }
            case 0 {
                let pow := 1
                base := mod(base, modulus)
                let next_limb_pointer := memory_exponent_pointer
                for { let limb_number := 0 } lt(limb_number, exponent_limbs) { limb_number := add(limb_number, ONE()) } {
                    let current_limb := mload(next_limb_pointer)
                    for { let i := 0 } gt(current_limb, ZERO()) { i := add(i, 1) } {
                        if eq(mod(current_limb, TWO()), ONE()) {
                            pow := mulmod(pow, base, modulus)
                        }
                        current_limb := shr(1, current_limb)
                        base := mulmod(base, base, modulus)
                    }
                    next_limb_pointer := add(next_limb_pointer, WORD_SIZE())
                }
                mstore(0, pow)
                let unpadding := sub(WORD_SIZE(), modulus_length)
                return(unpadding, modulus_length)
            }
		}
	}
}
