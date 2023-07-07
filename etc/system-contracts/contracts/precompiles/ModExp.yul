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

            ////////////////////////////////////////////////////////////////
            //                      FALLBACK
            ////////////////////////////////////////////////////////////////

            calldatacopy(0, 0, 32)
            let base_length := mload(0)

            calldatacopy(32, 32, 32)
            let exponent_length := mload(32)

            calldatacopy(64, 64, 32)
            let modulus_length := mload(64)

            let next_free_pointer := 96
            calldatacopy(add(96, sub(32, base_length)), next_free_pointer, base_length)
            let base := mload(next_free_pointer)

            next_free_pointer := add(next_free_pointer, base_length)
            calldatacopy(add(128, sub(32, exponent_length)), next_free_pointer, exponent_length)
            let exponent := mload(next_free_pointer)
            
            next_free_pointer := add(next_free_pointer, exponent_length)
            calldatacopy(add(160, sub(32, modulus_length)), next_free_pointer, modulus_length)
            let modulus := mload(next_free_pointer)

            // // base^0 % modulus = 1
            // if iszero(exponent) {
            //     mstore(0, ONE())
            //     return(0, modulus_length)
            // }

            // // base^exponent % 0 = 0
            // if iszero(modulus) {
            //     let s := add(add(base_length, exponent_length), modulus_length)
            //     mstore(s, ZERO())
            //     return(s, modulus_length)
            // }

            let pow := 1
            for { } gt(exponent, ZERO()) { } {
                    if eq(mod(exponent, 2), ONE()) {
                        pow := mulmod(pow, base, modulus)
                    }
                    exponent := shr(1, exponent)
                    base := mulmod(base, base, modulus)
            }

            next_free_pointer := add(next_free_pointer, modulus_length)
            mstore(next_free_pointer, pow)
            return(next_free_pointer, modulus_length)
		}
	}
}

// pow: 1 base: 2 exp: 3
// pow: 2 base: 4 exp: 1
// pow: 8 base: 16 exp: 0