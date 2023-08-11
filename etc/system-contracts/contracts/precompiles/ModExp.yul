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
                for { let limb_number := 0 } lt(limb_number, exponent_limbs) { limb_number := add(limb_number, ONE()) } {
                    let limb := mload(add(exponent_pointer, mul(32, limb_number)))
                    isZero := or(isZero, limb)
                    if isZero {
                        break
                    }
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

            let base_pointer := 96
            let base_padding := sub(32, base_length)
            let padded_base_pointer := add(96, base_padding)
            calldatacopy(padded_base_pointer, base_pointer, base_length)
            let base := mload(base_pointer)
            
            let calldata_exponent_pointer := add(base_pointer, base_length)
            let memory_exponent_pointer := add(base_pointer, 32)
            let exponent_limbs := ZERO()
            switch iszero(mod(exponent_length, 32))
            case 0 {
                exponent_limbs := add(div(exponent_length, 32), ONE())
            }
            case 1 {
                exponent_limbs := div(exponent_length, 32)
            }
            // The exponent expected length given the amount of limbs.
            let adjusted_exponent_length := mul(32, exponent_limbs)
            let next_limb_pointer := calldata_exponent_pointer
            for { let limb_number := 0 } lt(limb_number, exponent_limbs) { limb_number := add(limb_number, ONE()) } {
                // The msb of the leftmost limb could be one.
                // This left-pads with zeros the leftmost limbs to achieve 32 bytes.
                if iszero(limb_number) {
                    // The amount of zeros to left-pad.
                    let padding := sub(adjusted_exponent_length, exponent_length)
                    // This is either 0 or > 0 if there are any zeros to pad.
                    let padded_exponent_pointer := add(memory_exponent_pointer, padding)
                    let amount_of_bytes_for_first_limb := sub(32, padding)
                    calldatacopy(padded_exponent_pointer, calldata_exponent_pointer, amount_of_bytes_for_first_limb)
                    next_limb_pointer := add(calldata_exponent_pointer, amount_of_bytes_for_first_limb)
                    continue
                }
                calldatacopy(next_limb_pointer, next_limb_pointer, 32)
                next_limb_pointer := add(next_limb_pointer, 32)
            }

            let calldata_modulus_pointer := add(calldata_exponent_pointer, exponent_length)
            let memory_modulus_pointer := add(memory_exponent_pointer, adjusted_exponent_length)
            calldatacopy(add(memory_modulus_pointer, sub(32, modulus_length)), calldata_modulus_pointer, modulus_length)

            let modulus := mload(memory_modulus_pointer)

            // 1^exponent % modulus = 1
            if eq(base, ONE()) {
                mstore(0, ONE())
                let unpadding := sub(32, modulus_length)
                return(unpadding, modulus_length)
            }

            // base^exponent % 0 = 0
            if iszero(modulus) {
                mstore(0, ZERO())
                let unpadding := sub(32, modulus_length)
                return(unpadding, modulus_length)
            }

            // base^0 % modulus = 1
            if exponentIsZero(exponent_length, add(base_pointer, base_length)) {
                mstore(0, ONE())
                let unpadding := sub(32, modulus_length)
                return(unpadding, modulus_length)
            }

            // 0^exponent % modulus = 0
            if eq(base, ZERO()) {
                mstore(0, ZERO())
                let unpadding := sub(32, modulus_length)
                return(unpadding, modulus_length)
            }

            console_log(0x600, exponent_limbs)

            if eq(exponent_limbs, ONE()) {
                let pow := 1
                // If we have one limb, then the exponent has 32 bytes and it is
                // located in 0x
                let exponent := mload(128)
                base := mod(base, modulus)
                for { let i := 0 } gt(exponent, ZERO()) { i := add(i, 1) } {
                    if eq(mod(exponent, TWO()), ONE()) {
                        pow := mulmod(pow, base, modulus)
                    }
                    exponent := shr(1, exponent)
                    base := mulmod(base, base, modulus)
                }
    
                mstore(0, pow)
                let unpadding := sub(32, modulus_length)
                return(unpadding, modulus_length)
            }

            // let exponent_limbs := add(div(exponent_length, 32), ONE())

            // let pow := 1
            // base := mod(base, modulus)
            // for { let i := 0 } gt(exponent, ZERO()) { i := add(i, 1) } {
            //     if eq(mod(exponent, TWO()), ONE()) {
            //         pow := mulmod(pow, base, modulus)
            //     }
            //     exponent := shr(1, exponent)
            //     base := mulmod(base, base, modulus)
            // }

            // mstore(0, pow)
            // return(sub(add(0, 32), modulus_length), modulus_length)
		}
	}
}
