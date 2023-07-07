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

			function THREE() -> three {
                three := 0x3
            }

			function NINE() -> nine {
                nine := 0x9
            }

			function Q() -> q {
				q := 21888242871839275222246405745257275088548364400416034343698204186575808495617
			}

			function P() -> p {
				p := 21888242871839275222246405745257275088696311157297823662689037894645226208583
			}

			function G1_GENERATOR() -> x, y {
				x := 1
				y := 2
			}

			function G2_GENERATOR(i) -> x, y {
				x := add(mul(11559732032986387107991004021392285783925812861821192530917403151452391805634, i), 10857046999023057135944570762232829481370756359578518086990519993285655852781)
				y := add(mul(4082367875863433681332203403145435568316851327593401208105741076214120093531, i), 8495653923123431417604973247489272438418190587263600148770280649306958101930)
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

			function powmod(
				base,
				exponent,
				modulus,
			) -> pow {
				pow := 1
				let base := mod(base, modulus)
				let exponent := exponent
				for { } gt(exponent, ZERO()) { } {
					if eq(mod(exponent, 2), ONE()) {
						pow := mulmod(pow, base, modulus)
					}
					exponent := shr(1, exponent)
					base := mulmod(base, base, modulus)
				}
			}

			function invmod(base, modulus) -> inv {
				inv := powmod(base, sub(modulus, 2), modulus)
			}

			function divmod(dividend, divisor, modulus) -> quotient {
				quotient := mulmod(dividend, invmod(divisor, modulus), modulus)
			}

			// G1 -> Y^2 = X^3 + 3
			function pointIsOnG1(x, y) -> ret {
				let y_squared := mulmod(y, y, P())
				let x_squared := mulmod(x, x, P())
				let x_qubed := mulmod(x_squared, x, P())
				let x_qubed_plus_three := addmod(x_qubed, THREE(), P())

				ret := eq(y_squared, x_qubed_plus_three)
			}

			// G2 -> Y^2 = X^3 + 3/(i+9)
			function pointIsOnG2(x, y, i) -> ret {
				let y_squared := mulmod(y, y, P())
				let x_squared := mulmod(x, x, P())
				let x_qubed := mulmod(x_squared, x, P())

				let i_times_nine := addmod(i, NINE(), P())
				let three_over_i_times_nine := divmod(THREE(), i_times_nine, P())

				let x_qubed_plus_three_over_i_times_nine := addmod(x_qubed, three_over_i_times_nine, P())

				ret := eq(y_squared, x_qubed_plus_three_over_i_times_nine)
			}

			function log_P1(a) {

			}

			function log_P2(b) {

			}

			function checkPairing(...) {
				
			}

			////////////////////////////////////////////////////////////////
            //                      FALLBACK
            ////////////////////////////////////////////////////////////////

		  	let inputSize := calldatasize()

			// Empty input is valid and results in returning one.
		  	if eq(inputSize, ZERO()) {
				mstore(0, ONE())
				return(0, 32)
			}

			// If the input length is not a multiple of 192, the call fails.
            if iszero(eq(mod(inputSize, 192), 0)) {
                // Bad pairing input
				burnGas()
                revert(0, 0)
            }

			// let k := div(inputSize, 192)

			for { let i := 0 } lt(i, inputSize) { i := add(i, 192) } {
				/* G1 */
				calldatacopy(i, i, 32)
				calldatacopy(add(i, 32), add(i, 32), 32)

				let g1_x := mload(i)
				let g1_y := mload(add(i, 32))

				if iszero(pointIsOnG1(g1_x, g1_y)) {
					burnGas()
					revert(0, 0)
				}

				/* G2 */
				calldatacopy(add(i, 64), add(i, 64), 64) // x
				calldatacopy(add(i, 128), add(i, 128), 64) // y

				let g2_x := mload(add(i, 64))
				let g2_y := mload(add(i, 128))

				if iszero(pointIsOnG2(g2_x, g2_y, i)) {
					burnGas()
					revert(0, 0)
				}
			}

			// Return one if log_P1(a1) * log_P2(b1) + ... + log_P1(ak) * log_P2(bk) = 0
			if iszero(checkPairing(...)) {
				mstore(0, ONE())
				return(0, 32)
			}

			// Return zero otherwise
			mstore(0, ZERO())
			return(0, 32)
		}
	}
}
