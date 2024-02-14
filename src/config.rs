use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "APPLICATION_URL")]
    pub application_url: String,

    #[envconfig(from = "APPLICATION_PORT")]
    pub application_port: String,

    #[envconfig(from = "KEYCLOAK_URL")]
    pub keycloak_url: String,

    #[envconfig(from = "KEYCLOAK_REALM")]
    pub keycloak_realm: String,

    #[envconfig(from = "KEYCLOAK_CLIENT_NAME")]
    pub keycloak_client_name: String,

    #[envconfig(from = "KEYCLOAK_CLIENT_SECRET")]
    pub keycloak_client_secret: String,

    #[envconfig(from = "KEYCLOAK_ISSUER_URL")]
    pub keycloak_issuer_url: String,

    #[envconfig(from = "KEYCLOAK_INTROSPECT_URL")]
    pub keycloak_introspect_url: String,

    #[envconfig(from = "KEYCLOAK_WELL_KNOWN_CONFIG_URL")]
    pub keycloak_well_known_config_url: String,
}