object "Playground" {
    code { }
    object "Playground_deployed" {
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

            function THREE() -> three {
                three := 0x3
            }

            function MONTGOMERY_ONE() -> m_one {
                m_one := 15537367993719455909907449462855742678907882278146377936676643359958227611562
            }

            function MONTGOMERY_TWO() -> m_two {
                m_two := 9186493115599636597568493180454210269119453398994932210664248825271229014541
            }

            function MONTGOMERY_THREE() -> m_three {
                m_three := 2835618237479817285229536898052677859331024519843486484651854290584230417520
            }

            // Group order of alt_bn128, see https://eips.ethereum.org/EIPS/eip-196
            function ALT_BN128_GROUP_ORDER() -> ret {
                ret := 21888242871839275222246405745257275088696311157297823662689037894645226208583
            }

            function R2_MOD_ALT_BN128_GROUP_ORDER() -> ret {
                ret := 3096616502983703923843567936837374451735540968419076528771170197431451843209
            }

            function R3_MOD_ALT_BN128_GROUP_ORDER() -> ret {
                ret := 14921786541159648185948152738563080959093619838510245177710943249661917737183
            }

            function N_PRIME() -> ret {
                ret := 111032442853175714102588374283752698368366046808579839647964533820976443843465
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

            function invmod(base, modulus) -> inv {
                inv := powmod(base, sub(modulus, TWO()), modulus)
            }

            function powmod(
                base,
                exponent,
                modulus,
            ) -> pow {
                pow := 1
                let aux_exponent := exponent
                for { } gt(aux_exponent, ZERO()) { } {
                    if mod(aux_exponent, TWO()) {
                        pow := mulmod(pow, base, modulus)
                    }
                    aux_exponent := shr(1, aux_exponent)
                    base := mulmod(base, base, modulus)
                }
            }

            function overflowingAdd(augend, addend) -> sum, overflowed {
                sum := add(augend, addend)
                overflowed := or(lt(sum, augend), lt(sum, addend))
            }

            function getHighestHalfOfMultiplication(multiplicand, multiplier) -> ret {
                ret := verbatim_2i_1o("mul_high", multiplicand, multiplier)
            }

            // https://en.wikipedia.org/wiki/Montgomery_modular_multiplication//The_REDC_algorithm
            function REDC(lowest_half_of_T, higher_half_of_T) -> S {
                let q := mul(lowest_half_of_T, N_PRIME())
                let a_high := add(higher_half_of_T, getHighestHalfOfMultiplication(q, ALT_BN128_GROUP_ORDER()))
                let a_low, overflowed := overflowingAdd(lowest_half_of_T, mul(q, ALT_BN128_GROUP_ORDER()))
                if overflowed {
                    a_high := add(a_high, ONE())
                }
                S := a_high
                if iszero(lt(a_high, ALT_BN128_GROUP_ORDER())) {
                    S := sub(a_high, ALT_BN128_GROUP_ORDER())
                }
            }

            // Transforming into the Montgomery form -> REDC((a mod N)(R2 mod N))
            function intoMontgomeryForm(a) -> ret {
                    let higher_half_of_a := getHighestHalfOfMultiplication(mod(a, ALT_BN128_GROUP_ORDER()), R2_MOD_ALT_BN128_GROUP_ORDER())
                    let lowest_half_of_a := mul(mod(a, ALT_BN128_GROUP_ORDER()), R2_MOD_ALT_BN128_GROUP_ORDER())
                    ret := REDC(lowest_half_of_a, higher_half_of_a)
            }

            // Transforming out of the Montgomery form -> REDC(a * R mod N)
            function outOfMontgomeryForm(m) -> ret {
                    let higher_half_of_m := ZERO()
                    let lowest_half_of_m := m 
                    ret := REDC(lowest_half_of_m, higher_half_of_m)
            }

            // Multipling field elements in Montgomery form -> REDC((a * R mod N)(b * R mod N))
            function montgomeryMul(multiplicand, multiplier) -> ret {
                let higher_half_of_product := getHighestHalfOfMultiplication(multiplicand, multiplier)
                let lowest_half_of_product := mul(multiplicand, multiplier)
                ret := REDC(lowest_half_of_product, higher_half_of_product)
            }

            function montgomeryModExp(
                base,
                exponent
            ) -> pow {
                pow := MONTGOMERY_ONE()
                let aux_exponent := exponent
                for { } gt(aux_exponent, ZERO()) { } {
                    if mod(aux_exponent, 2) {
                            pow := montgomeryMul(pow, base)
                    }
                    aux_exponent := shr(1, aux_exponent)
                    base := montgomeryMul(base, base)
                }
            }

            function montgomeryModularInverse(a) -> invmod {
                let a_inv := invmod(a, ALT_BN128_GROUP_ORDER())
                let higher_half_of_inverse := getHighestHalfOfMultiplication(a_inv, R3_MOD_ALT_BN128_GROUP_ORDER())
                let lowest_half_of_inverse := mul(a_inv, R3_MOD_ALT_BN128_GROUP_ORDER())
                invmod := REDC(lowest_half_of_inverse, higher_half_of_inverse)
            }

            function montgomeryDiv(dividend, divisor) -> quotient {
                quotient := montgomeryMul(dividend, montgomeryModExp(divisor, sub(ALT_BN128_GROUP_ORDER(), TWO())))
            }

            let N := ALT_BN128_GROUP_ORDER()
            let a := THREE()
            let a_mont := intoMontgomeryForm(a)

            // a
            console_log(0x00, a)
            // a in montgomery form
            console_log(0x40, a_mont)
            // a in montgomery form into regular form
            console_log(0x80, outOfMontgomeryForm(a_mont))

            /* Addition */
            let sum_mont := addmod(a_mont, a_mont, N)
            console_log(0xc0, addmod(a, a, N))
            // a * a in montgomery form
            console_log(0x100, sum_mont)
            // a * a in montgomery form into montgomery form
            console_log(0x140, outOfMontgomeryForm(sum_mont))

            /* Multiplication */

            let prod_mont := montgomeryMul(a_mont, a_mont)
            console_log(0x180, mulmod(a, a, N))
            // a * a in montgomery form
            console_log(0x1c0, prod_mont)
            // a * a in montgomery form into montgomery form
            console_log(0x200, outOfMontgomeryForm(prod_mont))
        
            /* Modular Exponentiation */
            let pow_mont := montgomeryModExp(a_mont, 3)
            console_log(0x240, mulmod(a, mulmod(a, a, N), N))
            // a ** 3 in montgomery form
            console_log(0x280, pow_mont)
            // a ** 3 in montgomery form into montgomery form
            console_log(0x2c0, outOfMontgomeryForm(pow_mont))

            /* Modular Inverse */
            // a^-1
            console_log(0x300, invmod(a, N))
            let a_inv_mont := montgomeryModularInverse(a_mont)
            // a^-1 into montgomery form
            console_log(0x340, a_inv_mont)
            // a^-1 out of montgomery form
            console_log(0x3c0, outOfMontgomeryForm(a_inv_mont))

            /* Division */
            // a // a
            console_log(0x400, div(a, a))
            // a // a in montgomery form
            let div_mont := montgomeryMul(a_mont, a_inv_mont)
            console_log(0x440, div_mont)
            // a // a out of montgomery form
            console_log(0x480, outOfMontgomeryForm(div_mont))
        }
    }
}
