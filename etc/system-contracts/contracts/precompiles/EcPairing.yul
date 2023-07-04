object "EcPairing" {
	code { }
	object "EcPairing_deployed" {
		code {
            if not(eq(mod(calldatasize(), 0xc0), 0)) {
                // Bad pairing input
                revert(0,0)
            }
		}
	}
}
