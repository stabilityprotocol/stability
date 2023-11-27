# SlowMist Audit Report Summary - Stability Pallets

## Executive Summary

On 2023-10-23, the SlowMist security team conducted a "white box" security audit on the Stability pallets. They employed black box testing, grey box testing, and white box testing to ensure a thorough review from multiple perspectives.

## Project Overview

The audit targeted the Stability blockchain implemented in Substrate + Rust, focusing on several specific pallets (modules) within the codebase, as listed in the audit report.

## Findings and Actions Taken

### High Severity Vulnerabilities

1. **Arithmetic Accuracy Deviation Vulnerability**:

   - **Description**: Potential loss of precision or accuracy due to the use of `saturating_add`, `saturating_mul`, and `saturating_sub` in Rust.
   - **Action Taken**: Replaced with checked arithmetic functions (`checked_add`, `checked_mul`, `checked_sub`) to handle overflows gracefully.
   - **Status**: Fixed.

2. **Integer Overflow Audit**:

   - **Description**: Risks of integer overflow in numeric variables without proper overflow checks.
   - **Action Taken**: Implemented checked arithmetic functions.
   - **Status**: Fixed.

3. **Error Unhandle Audit (Division by Zero)**:
   - **Description**: Potential program panic due to division by zero in Rust.
   - **Action Taken**: Added checks for division by zero.
   - **Status**: Fixed.

### Low Severity Vulnerabilities

1. **Weights Audit (Unreasonable Pallet Weight)**:

   - **Description**: Operations having their weight set to 0, potentially leading to unreasonable resource allocation.
   - **Action Taken**: Reviewed and adjusted weights based on computational requirements.
   - **Status**: Fixed.

2. **Arithmetic Accuracy Deviation Vulnerability (Balance Precision Loss)**:
   - **Description**: Loss of balance precision when converting `U256` to `u128`.
   - **Action Taken**: Acknowledged as a known limitation; implemented fallback checks.
   - **Status**: Acknowledged.

### Suggested Improvements

1. **Unimplemented Function Logic**:

   - **Description**: Certain functions lacking full implementation.
   - **Action Taken**: Acknowledged and reviewed; these functions were mocked as they are not utilized in the current logic.
   - **Status**: Acknowledged.

2. **Node Crash Risk (Use of `panic!()`)**:

   - **Description**: Potential node crash due to the use of `panic!()` in certain functions.
   - **Action Taken**: Replaced `panic!()` with appropriate error handling.
   - **Status**: Fixed.

3. **Avoid Hardcoding Values**:
   - **Description**: Hardcoded Ethereum addresses in the code.
   - **Action Taken**: Refactored to use configuration files, environment variables, and parameterization for more flexibility.
   - **Status**: Fixed.

## Conclusion

The audit identified 4 high-risk, 4 low-risk, and 3 suggested vulnerability categories. The team has addressed most of the vulnerabilities, with some acknowledged due to their nature or current architecture limitations.

Find the full report [here](media/SlowMIst-Audit-November-2023.pdf).
