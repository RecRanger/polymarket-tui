//! Authentication state types

/// Login form field being edited
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginField {
    ApiKey,
    Secret,
    Passphrase,
    Address,
    // Optional cookie fields for favorites
    SessionCookie,
    SessionNonce,
    SessionAuthType,
}

#[allow(dead_code)]
impl LoginField {
    pub fn next(&self) -> Self {
        match self {
            LoginField::ApiKey => LoginField::Secret,
            LoginField::Secret => LoginField::Passphrase,
            LoginField::Passphrase => LoginField::Address,
            LoginField::Address => LoginField::SessionCookie,
            LoginField::SessionCookie => LoginField::SessionNonce,
            LoginField::SessionNonce => LoginField::SessionAuthType,
            LoginField::SessionAuthType => LoginField::ApiKey,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            LoginField::ApiKey => LoginField::SessionAuthType,
            LoginField::Secret => LoginField::ApiKey,
            LoginField::Passphrase => LoginField::Secret,
            LoginField::Address => LoginField::Passphrase,
            LoginField::SessionCookie => LoginField::Address,
            LoginField::SessionNonce => LoginField::SessionCookie,
            LoginField::SessionAuthType => LoginField::SessionNonce,
        }
    }
}

/// Login form state
#[derive(Debug, Clone)]
pub struct LoginFormState {
    pub api_key: String,
    pub secret: String,
    pub passphrase: String,
    pub address: String,
    // Optional cookie fields for favorites functionality
    pub session_cookie: String,
    pub session_nonce: String,
    pub session_auth_type: String,
    pub active_field: LoginField,
    pub error_message: Option<String>,
    pub is_validating: bool,
}

#[allow(dead_code)]
impl LoginFormState {
    pub fn new() -> Self {
        Self {
            api_key: String::new(),
            secret: String::new(),
            passphrase: String::new(),
            address: String::new(),
            session_cookie: String::new(),
            session_nonce: String::new(),
            session_auth_type: String::from("magic"), // Default to "magic"
            active_field: LoginField::ApiKey,
            error_message: None,
            is_validating: false,
        }
    }

    pub fn get_active_field_value(&self) -> &str {
        match self.active_field {
            LoginField::ApiKey => &self.api_key,
            LoginField::Secret => &self.secret,
            LoginField::Passphrase => &self.passphrase,
            LoginField::Address => &self.address,
            LoginField::SessionCookie => &self.session_cookie,
            LoginField::SessionNonce => &self.session_nonce,
            LoginField::SessionAuthType => &self.session_auth_type,
        }
    }

    pub fn add_char(&mut self, c: char) {
        match self.active_field {
            LoginField::ApiKey => self.api_key.push(c),
            LoginField::Secret => self.secret.push(c),
            LoginField::Passphrase => self.passphrase.push(c),
            LoginField::Address => self.address.push(c),
            LoginField::SessionCookie => self.session_cookie.push(c),
            LoginField::SessionNonce => self.session_nonce.push(c),
            LoginField::SessionAuthType => self.session_auth_type.push(c),
        }
        self.error_message = None;
    }

    pub fn delete_char(&mut self) {
        match self.active_field {
            LoginField::ApiKey => {
                self.api_key.pop();
            },
            LoginField::Secret => {
                self.secret.pop();
            },
            LoginField::Passphrase => {
                self.passphrase.pop();
            },
            LoginField::Address => {
                self.address.pop();
            },
            LoginField::SessionCookie => {
                self.session_cookie.pop();
            },
            LoginField::SessionNonce => {
                self.session_nonce.pop();
            },
            LoginField::SessionAuthType => {
                self.session_auth_type.pop();
            },
        }
        self.error_message = None;
    }

    pub fn clear(&mut self) {
        self.api_key.clear();
        self.secret.clear();
        self.passphrase.clear();
        self.address.clear();
        self.session_cookie.clear();
        self.session_nonce.clear();
        self.session_auth_type = String::from("magic"); // Reset to default
        self.active_field = LoginField::ApiKey;
        self.error_message = None;
        self.is_validating = false;
    }
}

/// User profile information from Polymarket
#[derive(Debug, Clone, Default)]
pub struct UserProfile {
    pub name: Option<String>,
    pub pseudonym: Option<String>,
    pub bio: Option<String>,
    pub profile_image: Option<String>,
}

/// User authentication state
#[derive(Debug, Clone)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub username: Option<String>,
    pub address: Option<String>,
    pub balance: Option<f64>,           // USDC cash balance
    pub portfolio_value: Option<f64>,   // Total portfolio value (positions)
    pub positions_count: Option<usize>, // Number of open positions
    pub unrealized_pnl: Option<f64>,    // Unrealized profit/loss
    pub realized_pnl: Option<f64>,      // Realized profit/loss
    pub profile: Option<UserProfile>,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            is_authenticated: false,
            username: None,
            address: None,
            balance: None,
            portfolio_value: None,
            positions_count: None,
            unrealized_pnl: None,
            realized_pnl: None,
            profile: None,
        }
    }

    pub fn display_name(&self) -> String {
        if let Some(ref name) = self.username {
            name.clone()
        } else if let Some(ref addr) = self.address {
            addr.clone()
        } else {
            "Unknown".to_string()
        }
    }
}
