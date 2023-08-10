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
            function unsafePackPrecompileParams(
                uint32_inputOffsetInWords,
                uint32_inputLengthInWords,
                uint32_outputOffsetInWords,
                uint32_outputLengthInWords,
                uint64_perPrecompileInterpreted
            ) -> rawParams {
                rawParams := uint32_inputOffsetInWords
                rawParams := or(rawParams, shl(32, uint32_inputLengthInWords))
                rawParams := or(rawParams, shl(64, uint32_outputOffsetInWords))
                rawParams := or(rawParams, shl(96, uint32_outputLengthInWords))
                rawParams := or(rawParams, shl(192, uint64_perPrecompileInterpreted))
            }

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

            ////////////////////////////////////////////////////////////////
            //                      FALLBACK
            ////////////////////////////////////////////////////////////////

            let calldataSize := calldatasize() 
            calldatacopy(0, 0, calldataSize)

            let base_length := calldataload(0)
            let exponent_length := calldataload(32)
            let modulus_length := calldataload(64)

            let base_pointer := 96
            calldatacopy(add(96, sub(32, base_length)), base_pointer, base_length)
            
            let exponent_pointer := add(base_pointer, base_length)
            let exponent_limbs := add(div(exponent_length, 32), ONE())
            for { let limb_number := 0 } lt(limb_number, exponent_limbs) { limb_number := add(limb_number, ONE()) } {
                // The msb of the left most limb could be one.
                if iszero(limb_number) {
                    let first_limb_length := add(exponent_pointer, sub(mul(32, exponent_limbs), 32))
                    calldatacopy(first_limb_length, first_limb_length, 32)
                    exponent_pointer := add(exponent_pointer, 32)
                    continue
                }
                calldatacopy(exponent_pointer, exponent_pointer, 32)
                exponent_pointer := add(exponent_pointer, 32)
            }
            
            let modulus_pointer := add(exponent_pointer, exponent_length)
            calldatacopy(add(160, sub(32, modulus_length)), modulus_pointer, modulus_length)

            let base := mload(base_pointer)
            let modulus := mload(modulus_pointer)

            // 1^exponent % modulus = 1
            if eq(base, ONE()) {
                mstore(192, ONE())
                return(sub(add(192, 32), modulus_length), modulus_length)
            }

            // base^exponent % 0 = 0
            if iszero(modulus) {
                mstore(192, ZERO())
                return(sub(add(192, 32), modulus_length), modulus_length)
            }

            // base^0 % modulus = 1
            if exponentIsZero(exponent_length, add(base_pointer, base_length)) {
                mstore(192, ONE())
                return(sub(add(192, 32), modulus_length), modulus_length)
            }

            // 0^exponent % modulus = 0
            if eq(base, ZERO()) {
                mstore(192, ZERO())
                return(sub(add(192, 32), modulus_length), modulus_length)
            }

            if eq(exponent_limbs, ONE()) {
                let pow := 1

                // We save the exponent from calldataSize pointer
                calldatacopy(calldataSize, add(add(96, base_length), sub(32, exponent_length)), calldataSize)
                let exponent := mload(calldataSize)

                base := mod(base, modulus)
                for { let i := 0 } gt(exponent, ZERO()) { i := add(i, 1) } {
                    if eq(mod(exponent, TWO()), ONE()) {
                        pow := mulmod(pow, base, modulus)
                    }
                    exponent := shr(1, exponent)
                    base := mulmod(base, base, modulus)
                }
    
                mstore(0, pow)
                return(sub(add(0, 32), modulus_length), modulus_length)
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
