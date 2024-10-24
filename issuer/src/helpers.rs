use super::*;
use hex::encode;
use wasm_bindgen::prelude::JsValue;
use web_sys::console;
use crate::{Field, CurrentNetwork};

pub trait Logger {
    fn log(&self, message: &str);
}

pub struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn log(&self, message: &str) {
        console::log_1(&JsValue::from_str(message));
    }
}

pub struct StdoutLogger;

impl Logger for StdoutLogger {
    fn log(&self, message: &str) {
        println!("{}", message);
    }
}

pub fn string_to_field(input_str: Option<String>) -> Result<Field<CurrentNetwork>, anyhow::Error> {
    // Convert the input string to a hex-encoded string
    if input_str.is_none() {
        return Ok(Field::<CurrentNetwork>::zero());
    };
    let string_value = input_str.ok_or(anyhow!("The input string was None"))?;
    let u128type = match string_value.as_str().parse::<u128>() {
        Ok(value) => value,
        Err(_) => {
            let hex_encoded = encode(string_value.as_str());
            // Attempt to parse the hex-encoded string into a u128
            u128::from_str_radix(&hex_encoded, 16)
                .map_err(|e| anyhow!("String to field conversion error: {}", e))?
        },
    };

    Ok(Field::<CurrentNetwork>::from_u128(u128type))

}

// Helper functions for various cryptographic and utility operations.
pub(crate) fn insert_to_map(map: &mut IndexMap<Identifier<CurrentNetwork>, Plaintext<CurrentNetwork>>, key: &str, value: Plaintext<CurrentNetwork>) -> Result<(), anyhow::Error> {
    let id = Identifier::from_str(key)
        .map_err(|e| anyhow!("{}: {}", format!("Can't convert {} to Identifier", key), e))?;
    map.insert(id, value);
    Ok(())
}

pub(crate) fn generate_message_with_addresses_and_fields(payload: Credential) -> Result<Value<CurrentNetwork>, anyhow::Error> {
    let mut map = IndexMap::with_capacity(3);

    insert_to_map(&mut map, "issuer", Plaintext::from(Literal::Address(payload.issuer)))?;
    insert_to_map(&mut map, "subject", Plaintext::from(Literal::Address(payload.subject)))?;
    insert_to_map(&mut map, "dob", Plaintext::from(Literal::U32(payload.dob)))?;
    insert_to_map(&mut map, "nationality", Plaintext::from(Literal::Field(payload.nationality)))?;
    insert_to_map(&mut map, "expiry", Plaintext::from(Literal::U32(payload.expiry)))?;

    Ok(Value::Plaintext(Plaintext::Struct(map, Default::default())))
}

pub(crate) fn create_hash(value: Value<CurrentNetwork>, algorithm: HashAlgorithm) -> Result<String, anyhow::Error> {
    let hash = match algorithm  {
        HashAlgorithm::POSEIDON2 => {
            let message = value.to_fields()
                .map_err(|e| anyhow!("Failed value to Fields conversion: {}", e))?;
            let hash = CurrentNetwork::hash_psd2(message.as_slice())
                .map_err(|e| anyhow!("Failed hash_psd2 conversion: {}", e))?;
            hash.to_string()
        }
        HashAlgorithm::BHP1024 => {
            let message = value.to_bits_le();
            let hash = CurrentNetwork::hash_bhp1024(message.as_slice())
                .map_err(|e| anyhow!("Failed hash_bhp1024 conversion: {}", e))?;
            hash.to_string()
        }
        HashAlgorithm::SHA3_256 => {
            let message = value.to_bits_le();
            let sha_bit_vec = CurrentNetwork::hash_sha3_256(message.as_slice())
                .map_err(|e| anyhow!("Failed hash_sha3_256 conversion: {}", e))?;
            let bhp_group = CurrentNetwork::hash_to_group_bhp256(sha_bit_vec.as_slice())
                .map_err(|e| anyhow!("Failed hash_to_group_bhp256 conversion: {}", e))?;
            let literal_group_from_bhp = Literal::Group(bhp_group);
            let casted_to_field = literal_group_from_bhp
                .cast_lossy(snarkvm_console::program::LiteralType::Field)
                .map_err(|e| anyhow!("Failed cast_lossy conversion: {}", e))?;

            casted_to_field.to_string()
        }
        HashAlgorithm::KECCAK256 => {
            let message = value.to_bits_le();
            let keccak_bit_vec = CurrentNetwork::hash_keccak256(message.as_slice())
                .map_err(|e| anyhow!("Failed hash_keccak256 conversion: {}", e))?;
            let bhp_group = CurrentNetwork::hash_to_group_bhp256(keccak_bit_vec.as_slice())
                .map_err(|e| anyhow!("Failed hash_to_group_bhp256 conversion: {}", e))?;
            let literal_group_from_bhp = Literal::Group(bhp_group);
            let casted_to_field = literal_group_from_bhp
                .cast_lossy(snarkvm_console::program::LiteralType::Field)
                .map_err(|e| anyhow!("Failed cast_lossy conversion: {}", e))?;

            casted_to_field.to_string()
        }
    };
    Ok(hash)
}

pub(crate) fn sign_message_with_private_key(
    private_key: &PrivateKey<CurrentNetwork>,
    message: &[Field<CurrentNetwork>],
    rng: &mut TestRng
) -> Result<(Signature<CurrentNetwork>, Scalar<CurrentNetwork>), anyhow::Error> {
    match Signature::<CurrentNetwork>::sign(private_key, message, rng) {
        Ok(signature) => {
            let nonce = Scalar::rand(rng);
            Ok((signature, nonce))
        }
        Err(_) => Err(anyhow::anyhow!("Failed to create signature")),
    }
}


pub(crate) fn verify_signature_with_address_and_message(
    signature: &Signature<CurrentNetwork>,
    address: &Address<CurrentNetwork>,
    message: &[Field<CurrentNetwork>]
) -> bool {
    signature.verify(address, message)
}

pub(crate) fn string_to_value(s: &str) -> Value<CurrentNetwork> {
    Value::<CurrentNetwork>::from_str(s).expect("Can't convert string to Value")
}

pub(crate) fn string_to_value_fields(s: &str) -> Vec<Field<CurrentNetwork>> {
    let value  = string_to_value(s);
    value.to_fields().expect("Can't convert value to fields")
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_field_with_valid_u128() {
        let input_str = Some("12345".to_string());
        let result = string_to_field(input_str);
        assert!(result.is_ok());
        let field = result.unwrap();
        assert_eq!(field.to_string(), "12345field");
    }

    #[test]
    fn test_string_to_field_with_valid_hex_encoded_string() {
        let input_str = Some("American".to_string()); // "test" in hex
        let result = string_to_field(input_str);
        assert!(result.is_ok());
        let field = result.unwrap();
        assert_eq!(field.to_string(), "4714535926995575150field");
    }
}
