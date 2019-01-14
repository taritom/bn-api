

#[derive(Deserialize, Clone, Debug)]
pub struct SendGridError {
    errors: Vec<ErrorDetail>
}

#[derive(Deserialize, Clone, Debug)]
pub struct ErrorDetail {
    message: String
}

