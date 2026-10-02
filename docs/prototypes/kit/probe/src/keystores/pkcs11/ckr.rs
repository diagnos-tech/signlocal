//! Raw `CKR_*` return codes and names, which `cryptoki` hides behind its
//! `RvError` enum.
//!
//! Errors reach the user (and the proof reports) as "C_Login failed with
//! 0xa0: CKR_PIN_INCORRECT"; the numbers are what vendor documentation and
//! support tickets quote, so they must survive the translation.

use cryptoki::error::RvError;

/// The numeric value and the spec name of a return code.
pub fn describe(error: RvError) -> (i64, &'static str) {
    match error {
        RvError::Cancel => (0x1, "CKR_CANCEL"),
        RvError::HostMemory => (0x2, "CKR_HOST_MEMORY"),
        RvError::SlotIdInvalid => (0x3, "CKR_SLOT_ID_INVALID"),
        RvError::GeneralError => (0x5, "CKR_GENERAL_ERROR"),
        RvError::FunctionFailed => (0x6, "CKR_FUNCTION_FAILED"),
        RvError::ArgumentsBad => (0x7, "CKR_ARGUMENTS_BAD"),
        RvError::NoEvent => (0x8, "CKR_NO_EVENT"),
        RvError::NeedToCreateThreads => (0x9, "CKR_NEED_TO_CREATE_THREADS"),
        RvError::CantLock => (0xa, "CKR_CANT_LOCK"),
        RvError::AttributeReadOnly => (0x10, "CKR_ATTRIBUTE_READ_ONLY"),
        RvError::AttributeSensitive => (0x11, "CKR_ATTRIBUTE_SENSITIVE"),
        RvError::AttributeTypeInvalid => (0x12, "CKR_ATTRIBUTE_TYPE_INVALID"),
        RvError::AttributeValueInvalid => (0x13, "CKR_ATTRIBUTE_VALUE_INVALID"),
        RvError::ActionProhibited => (0x1b, "CKR_ACTION_PROHIBITED"),
        RvError::DataInvalid => (0x20, "CKR_DATA_INVALID"),
        RvError::DataLenRange => (0x21, "CKR_DATA_LEN_RANGE"),
        RvError::DeviceError => (0x30, "CKR_DEVICE_ERROR"),
        RvError::DeviceMemory => (0x31, "CKR_DEVICE_MEMORY"),
        RvError::DeviceRemoved => (0x32, "CKR_DEVICE_REMOVED"),
        RvError::EncryptedDataInvalid => (0x40, "CKR_ENCRYPTED_DATA_INVALID"),
        RvError::EncryptedDataLenRange => (0x41, "CKR_ENCRYPTED_DATA_LEN_RANGE"),
        RvError::FunctionCanceled => (0x50, "CKR_FUNCTION_CANCELED"),
        RvError::FunctionNotParallel => (0x51, "CKR_FUNCTION_NOT_PARALLEL"),
        RvError::FunctionNotSupported => (0x54, "CKR_FUNCTION_NOT_SUPPORTED"),
        RvError::CurveNotSupported => (0x140, "CKR_CURVE_NOT_SUPPORTED"),
        RvError::KeyHandleInvalid => (0x60, "CKR_KEY_HANDLE_INVALID"),
        RvError::KeySizeRange => (0x62, "CKR_KEY_SIZE_RANGE"),
        RvError::KeyTypeInconsistent => (0x63, "CKR_KEY_TYPE_INCONSISTENT"),
        RvError::KeyNotNeeded => (0x64, "CKR_KEY_NOT_NEEDED"),
        RvError::KeyChanged => (0x65, "CKR_KEY_CHANGED"),
        RvError::KeyNeeded => (0x66, "CKR_KEY_NEEDED"),
        RvError::KeyIndigestible => (0x67, "CKR_KEY_INDIGESTIBLE"),
        RvError::KeyFunctionNotPermitted => (0x68, "CKR_KEY_FUNCTION_NOT_PERMITTED"),
        RvError::KeyNotWrappable => (0x69, "CKR_KEY_NOT_WRAPPABLE"),
        RvError::KeyUnextractable => (0x6a, "CKR_KEY_UNEXTRACTABLE"),
        RvError::MechanismInvalid => (0x70, "CKR_MECHANISM_INVALID"),
        RvError::MechanismParamInvalid => (0x71, "CKR_MECHANISM_PARAM_INVALID"),
        RvError::ObjectHandleInvalid => (0x82, "CKR_OBJECT_HANDLE_INVALID"),
        RvError::OperationActive => (0x90, "CKR_OPERATION_ACTIVE"),
        RvError::OperationNotInitialized => (0x91, "CKR_OPERATION_NOT_INITIALIZED"),
        RvError::PinIncorrect => (0xa0, "CKR_PIN_INCORRECT"),
        RvError::PinInvalid => (0xa1, "CKR_PIN_INVALID"),
        RvError::PinLenRange => (0xa2, "CKR_PIN_LEN_RANGE"),
        RvError::PinExpired => (0xa3, "CKR_PIN_EXPIRED"),
        RvError::PinLocked => (0xa4, "CKR_PIN_LOCKED"),
        RvError::SessionClosed => (0xb0, "CKR_SESSION_CLOSED"),
        RvError::SessionCount => (0xb1, "CKR_SESSION_COUNT"),
        RvError::SessionHandleInvalid => (0xb3, "CKR_SESSION_HANDLE_INVALID"),
        RvError::SessionParallelNotSupported => (0xb4, "CKR_SESSION_PARALLEL_NOT_SUPPORTED"),
        RvError::SessionReadOnly => (0xb5, "CKR_SESSION_READ_ONLY"),
        RvError::SessionExists => (0xb6, "CKR_SESSION_EXISTS"),
        RvError::SessionReadOnlyExists => (0xb7, "CKR_SESSION_READ_ONLY_EXISTS"),
        RvError::SessionReadWriteSoExists => (0xb8, "CKR_SESSION_READ_WRITE_SO_EXISTS"),
        RvError::SignatureInvalid => (0xc0, "CKR_SIGNATURE_INVALID"),
        RvError::SignatureLenRange => (0xc1, "CKR_SIGNATURE_LEN_RANGE"),
        RvError::TemplateIncomplete => (0xd0, "CKR_TEMPLATE_INCOMPLETE"),
        RvError::TemplateInconsistent => (0xd1, "CKR_TEMPLATE_INCONSISTENT"),
        RvError::TokenNotPresent => (0xe0, "CKR_TOKEN_NOT_PRESENT"),
        RvError::TokenNotRecognized => (0xe1, "CKR_TOKEN_NOT_RECOGNIZED"),
        RvError::TokenWriteProtected => (0xe2, "CKR_TOKEN_WRITE_PROTECTED"),
        RvError::UnwrappingKeyHandleInvalid => (0xf0, "CKR_UNWRAPPING_KEY_HANDLE_INVALID"),
        RvError::UnwrappingKeySizeRange => (0xf1, "CKR_UNWRAPPING_KEY_SIZE_RANGE"),
        RvError::UnwrappingKeyTypeInconsistent => (0xf2, "CKR_UNWRAPPING_KEY_TYPE_INCONSISTENT"),
        RvError::UserAlreadyLoggedIn => (0x100, "CKR_USER_ALREADY_LOGGED_IN"),
        RvError::UserNotLoggedIn => (0x101, "CKR_USER_NOT_LOGGED_IN"),
        RvError::UserPinNotInitialized => (0x102, "CKR_USER_PIN_NOT_INITIALIZED"),
        RvError::UserTypeInvalid => (0x103, "CKR_USER_TYPE_INVALID"),
        RvError::UserAnotherAlreadyLoggedIn => (0x104, "CKR_USER_ANOTHER_ALREADY_LOGGED_IN"),
        RvError::UserTooManyTypes => (0x105, "CKR_USER_TOO_MANY_TYPES"),
        RvError::WrappedKeyInvalid => (0x110, "CKR_WRAPPED_KEY_INVALID"),
        RvError::WrappedKeyLenRange => (0x112, "CKR_WRAPPED_KEY_LEN_RANGE"),
        RvError::WrappingKeyHandleInvalid => (0x113, "CKR_WRAPPING_KEY_HANDLE_INVALID"),
        RvError::WrappingKeySizeRange => (0x114, "CKR_WRAPPING_KEY_SIZE_RANGE"),
        RvError::WrappingKeyTypeInconsistent => (0x115, "CKR_WRAPPING_KEY_TYPE_INCONSISTENT"),
        RvError::RandomSeedNotSupported => (0x120, "CKR_RANDOM_SEED_NOT_SUPPORTED"),
        RvError::RandomNoRng => (0x121, "CKR_RANDOM_NO_RNG"),
        RvError::DomainParamsInvalid => (0x130, "CKR_DOMAIN_PARAMS_INVALID"),
        RvError::BufferTooSmall => (0x150, "CKR_BUFFER_TOO_SMALL"),
        RvError::SavedStateInvalid => (0x160, "CKR_SAVED_STATE_INVALID"),
        RvError::InformationSensitive => (0x170, "CKR_INFORMATION_SENSITIVE"),
        RvError::StateUnsaveable => (0x180, "CKR_STATE_UNSAVEABLE"),
        RvError::CryptokiNotInitialized => (0x190, "CKR_CRYPTOKI_NOT_INITIALIZED"),
        RvError::CryptokiAlreadyInitialized => (0x191, "CKR_CRYPTOKI_ALREADY_INITIALIZED"),
        RvError::MutexBad => (0x1a0, "CKR_MUTEX_BAD"),
        RvError::MutexNotLocked => (0x1a1, "CKR_MUTEX_NOT_LOCKED"),
        RvError::NewPinMode => (0x1b0, "CKR_NEW_PIN_MODE"),
        RvError::NextOtp => (0x1b1, "CKR_NEXT_OTP"),
        RvError::ExceededMaxIterations => (0x1b5, "CKR_EXCEEDED_MAX_ITERATIONS"),
        RvError::FipsSelfTestFailed => (0x1b6, "CKR_FIPS_SELF_TEST_FAILED"),
        RvError::LibraryLoadFailed => (0x1b7, "CKR_LIBRARY_LOAD_FAILED"),
        RvError::PinTooWeak => (0x1b8, "CKR_PIN_TOO_WEAK"),
        RvError::PublicKeyInvalid => (0x1b9, "CKR_PUBLIC_KEY_INVALID"),
        RvError::FunctionRejected => (0x200, "CKR_FUNCTION_REJECTED"),
        RvError::VendorDefined(code) => (from_ck_rv(code), "CKR_VENDOR_DEFINED"),
        RvError::UnknownErrorCode(code) => (from_ck_rv(code), "CKR_UNKNOWN"),
    }
}

fn from_ck_rv(code: impl TryInto<i64>) -> i64 {
    code.try_into().unwrap_or(i64::MAX)
}

/// A short next step for the codes whose name alone does not tell the user
/// what to do.
pub fn hint(error: RvError) -> Option<&'static str> {
    Some(match error {
        RvError::PinExpired => "the PIN must be changed with the vendor's tool",
        RvError::UserPinNotInitialized => "the token has no user PIN yet",
        RvError::TokenNotRecognized => "the module does not recognize this card",
        RvError::UserAnotherAlreadyLoggedIn => "another user is logged in to this token",
        RvError::DeviceRemoved => "the token was removed",
        RvError::MechanismInvalid => "the token does not offer this signing mechanism",
        RvError::KeyFunctionNotPermitted => "the key is not allowed to sign",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_match_the_specification() {
        assert_eq!(describe(RvError::PinIncorrect), (0xa0, "CKR_PIN_INCORRECT"));
        assert_eq!(describe(RvError::PinLocked), (0xa4, "CKR_PIN_LOCKED"));
        assert_eq!(
            describe(RvError::UserNotLoggedIn),
            (0x101, "CKR_USER_NOT_LOGGED_IN")
        );
        assert_eq!(
            describe(RvError::FunctionRejected),
            (0x200, "CKR_FUNCTION_REJECTED")
        );
    }

    #[test]
    fn vendor_codes_keep_their_value() {
        assert_eq!(
            describe(RvError::VendorDefined(0x8000_0123)),
            (0x8000_0123, "CKR_VENDOR_DEFINED")
        );
    }
}
