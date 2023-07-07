object "EcPairing" {
	code { }
	object "EcPairing_deployed" {
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

			// ////////////////////////////////////////////////////////////////
			//                      HELPER FUNCTIONS
			// ////////////////////////////////////////////////////////////////

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

			function burnGas() {
				let precompileParams := unsafePackPrecompileParams(
					  0, // input offset in words
					  3, // input length in words (x, y, scalar)
					  0, // output offset in words
					  2, // output length in words (x2, y2)
					  0  // No special meaning
				)
				let gasToPay := gas()
	
				// Precompiles that do not have a circuit counterpart
				// will burn the provided gas by calling this function.
				precompileCall(precompileParams, gasToPay)
		  }

			////////////////////////////////////////////////////////////////
            //                      FALLBACK
            ////////////////////////////////////////////////////////////////

            if iszero(eq(mod(calldatasize(), 192), 0)) {
                // Bad pairing input
				burnGas()
                revert(0,0)
            }
		}
	}
}
