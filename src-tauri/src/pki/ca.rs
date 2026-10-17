use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DistinguishedName, DnType,
    ExtendedKeyUsagePurpose, IsCa, KeyUsagePurpose, SanType,
};
use std::path::Path;

/// Manages the local Root CA and wildcard *.test certificate generation
pub struct LocalCa {
    ca_cert_pem: String,
    ca_key_pem: String,
    ca_cert_dir: String,
}

impl LocalCa {
    /// Initialize or load the local Root CA from disk
    pub fn init(ssl_dir: &str) -> Result<Self, String> {
        std::fs::create_dir_all(ssl_dir)
            .map_err(|e| format!("Cannot create SSL dir: {}", e))?;

        let cert_path = format!("{}/ca.crt", ssl_dir);
        let key_path = format!("{}/ca.key", ssl_dir);

        if Path::new(&cert_path).exists() && Path::new(&key_path).exists() {
            let ca_cert_pem = std::fs::read_to_string(&cert_path)
                .map_err(|e| format!("Read CA cert: {}", e))?;
            let ca_key_pem = std::fs::read_to_string(&key_path)
                .map_err(|e| format!("Read CA key: {}", e))?;
            log::info!("Loaded existing Local Root CA from {}", ssl_dir);
            return Ok(Self { ca_cert_pem, ca_key_pem, ca_cert_dir: ssl_dir.to_string() });
        }

        // Generate a new Root CA
        let mut params = CertificateParams::default();
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "Laradoc Studio Local CA");
        dn.push(DnType::OrganizationName, "Laradoc Studio");
        dn.push(DnType::CountryName, "US");
        params.distinguished_name = dn;
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
            KeyUsagePurpose::DigitalSignature,
        ];

        // Valid for 10 years
        params.not_before = rcgen::date_time_ymd(2024, 1, 1);
        params.not_after = rcgen::date_time_ymd(2034, 1, 1);

        let ca = Certificate::from_params(params)
            .map_err(|e| format!("CA generation failed: {}", e))?;

        let ca_cert_pem = ca.serialize_pem()
            .map_err(|e| format!("Serialize CA cert: {}", e))?;
        let ca_key_pem = ca.serialize_private_key_pem();

        // Save to disk
        std::fs::write(&cert_path, &ca_cert_pem)
            .map_err(|e| format!("Write CA cert: {}", e))?;
        std::fs::write(&key_path, &ca_key_pem)
            .map_err(|e| format!("Write CA key: {}", e))?;

        log::info!("Generated new Local Root CA at {}", ssl_dir);

        Ok(Self { ca_cert_pem, ca_key_pem, ca_cert_dir: ssl_dir.to_string() })
    }

    /// Get path to the CA certificate
    pub fn cert_path(&self) -> String {
        format!("{}/ca.crt", self.ca_cert_dir)
    }

    /// Generate a wildcard *.test certificate signed by this CA
    pub fn generate_wildcard_test(&self) -> Result<(String, String), String> {
        let cert_path = format!("{}/wildcard.test.crt", self.ca_cert_dir);
        let key_path = format!("{}/wildcard.test.key", self.ca_cert_dir);

        if Path::new(&cert_path).exists() && Path::new(&key_path).exists() {
            let cert = std::fs::read_to_string(&cert_path)
                .map_err(|e| format!("Read wildcard cert: {}", e))?;
            let key = std::fs::read_to_string(&key_path)
                .map_err(|e| format!("Read wildcard key: {}", e))?;
            return Ok((cert, key));
        }

        // Load CA for signing
        let ca_params = CertificateParams::from_ca_cert_pem(&self.ca_cert_pem, &self.ca_key_pem)
            .map_err(|e| format!("Load CA params: {}", e))?;
        let ca_cert = Certificate::from_params(ca_params)
            .map_err(|e| format!("Rebuild CA: {}", e))?;

        // Wildcard params
        let mut params = CertificateParams::default();
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "*.test");
        params.distinguished_name = dn;
        params.subject_alt_names = vec![
            SanType::DnsName("*.test".into()),
            SanType::DnsName("localhost".into()),
            SanType::IpAddress(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
        ];
        params.extended_key_usages = vec![
            ExtendedKeyUsagePurpose::ServerAuth,
        ];
        params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
        params.not_before = rcgen::date_time_ymd(2024, 1, 1);
        params.not_after = rcgen::date_time_ymd(2034, 1, 1);

        let cert = Certificate::from_params(params)
            .map_err(|e| format!("Wildcard cert gen failed: {}", e))?;

        let cert_pem = cert
            .serialize_pem_with_signer(&ca_cert)
            .map_err(|e| format!("Sign wildcard cert: {}", e))?;
        let key_pem = cert.serialize_private_key_pem();

        std::fs::write(&cert_path, &cert_pem)
            .map_err(|e| format!("Write wildcard cert: {}", e))?;
        std::fs::write(&key_path, &key_pem)
            .map_err(|e| format!("Write wildcard key: {}", e))?;

        log::info!("Generated wildcard *.test certificate");
        Ok((cert_pem, key_pem))
    }
}
