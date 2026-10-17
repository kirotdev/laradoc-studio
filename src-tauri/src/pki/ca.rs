use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType,
    ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose,
};
use std::path::Path;

/// Manages the local Root CA and wildcard *.test certificate generation
#[allow(dead_code)]
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
        let mut params = CertificateParams::new(Vec::<String>::new())
            .map_err(|e| format!("CA params failed: {}", e))?;
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

        let key_pair = KeyPair::generate()
            .map_err(|e| format!("CA key generation failed: {}", e))?;

        let ca = params.self_signed(&key_pair)
            .map_err(|e| format!("CA generation failed: {}", e))?;

        let ca_cert_pem = ca.pem();
        let ca_key_pem = key_pair.serialize_pem();

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

        // Wildcard params with SANs
        let mut params = CertificateParams::new(vec![
            "*.test".to_string(),
            "localhost".to_string(),
        ])
        .map_err(|e| format!("Wildcard params error: {}", e))?;

        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "*.test");
        params.distinguished_name = dn;
        params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
        params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
        params.not_before = rcgen::date_time_ymd(2024, 1, 1);
        params.not_after = rcgen::date_time_ymd(2034, 1, 1);

        let wildcard_key = KeyPair::generate()
            .map_err(|e| format!("Wildcard key error: {}", e))?;

        // Rebuild CA cert representation for signing
        let mut ca_params = CertificateParams::new(Vec::<String>::new())
            .map_err(|e| format!("CA params error: {}", e))?;
        let mut ca_dn = DistinguishedName::new();
        ca_dn.push(DnType::CommonName, "Laradoc Studio Local CA");
        ca_dn.push(DnType::OrganizationName, "Laradoc Studio");
        ca_dn.push(DnType::CountryName, "US");
        ca_params.distinguished_name = ca_dn;
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
            KeyUsagePurpose::DigitalSignature,
        ];

        let ca_key = KeyPair::from_pem(&self.ca_key_pem)
            .map_err(|e| format!("Load CA key failed: {}", e))?;
        let ca_cert = ca_params.self_signed(&ca_key)
            .map_err(|e| format!("Self-sign CA failed: {}", e))?;

        let cert = params
            .signed_by(&wildcard_key, &ca_cert, &ca_key)
            .map_err(|e| format!("Sign wildcard cert failed: {}", e))?;

        let cert_pem = cert.pem();
        let key_pem = wildcard_key.serialize_pem();

        std::fs::write(&cert_path, &cert_pem)
            .map_err(|e| format!("Write wildcard cert: {}", e))?;
        std::fs::write(&key_path, &key_pem)
            .map_err(|e| format!("Write wildcard key: {}", e))?;

        log::info!("Generated wildcard *.test certificate");
        Ok((cert_pem, key_pem))
    }
}
