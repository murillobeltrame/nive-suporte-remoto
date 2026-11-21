// Personalização do Suporte Nive: servidor próprio, nome e aceite só por clique.
// Fica em módulo separado para que as atualizações vindas do RustDesk original
// entrem sem conflito; o único gancho é a chamada em `common::load_custom_client`.

use base::config::keys;
use hbb_common::config;

pub const APP_NAME: &str = "Suporte Nive";
pub const SERVER: &str = "suporte.nivesistemas.com.br";
pub const PUBLIC_KEY: &str = "cjUlV4XJm44hHYFjMfJwpihw+67PvQbKT4qdSKJU6s4=";

pub fn apply() {
    *config::APP_NAME.write().unwrap() = APP_NAME.to_owned();

    // Override: o usuário não consegue trocar pela tela de configurações.
    let mut settings = config::OVERWRITE_SETTINGS.write().unwrap();
    for (k, v) in [
        (keys::OPTION_CUSTOM_RENDEZVOUS_SERVER, SERVER),
        (keys::OPTION_RELAY_SERVER, SERVER),
        (keys::OPTION_KEY, PUBLIC_KEY),
        // Sem senha: toda conexão precisa do clique em "Aceitar" no computador do cliente.
        (keys::OPTION_APPROVE_MODE, "click"),
        (keys::OPTION_VERIFICATION_METHOD, "use-temporary-password"),
        (keys::OPTION_ALLOW_AUTO_UPDATE, "N"),
    ] {
        settings.insert(k.to_owned(), v.to_owned());
    }
    drop(settings);

    // A atualização do RustDesk original trocaria o servidor e o nome da Nive.
    config::OVERWRITE_LOCAL_SETTINGS
        .write()
        .unwrap()
        .insert(keys::OPTION_ENABLE_CHECK_UPDATE.to_owned(), "N".to_owned());

    let mut builtin = config::BUILTIN_SETTINGS.write().unwrap();
    for k in [
        keys::OPTION_HIDE_SERVER_SETTINGS,
        keys::OPTION_HIDE_PROXY_SETTINGS,
        keys::OPTION_HIDE_HELP_CARDS,
    ] {
        builtin.insert(k.to_owned(), "Y".to_owned());
    }
}

const CHECKIN_URL: &str = "https://erp.nivesistemas.com.br/api/public/suporte-remoto/checkin";
const CHECKIN_INTERVAL_SECS: f32 = 60.;

/// O ERP entrega o arquivo como `SuporteNive_<código da loja>.exe`. O Windows pode
/// acrescentar " (1)" em downloads repetidos, por isso só os caracteres do código contam.
fn parse_store_token(exe_name: &str) -> Option<String> {
    let file = exe_name.rsplit(['/', '\\']).next()?;
    let (_, rest) = file.split_once('_')?;
    let token: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    (token.len() >= 16).then_some(token)
}

fn store_token() -> Option<String> {
    // No executável empacotado, o nome original chega por esta variável.
    let name = std::env::var("RUSTDESK_APPNAME").ok().or_else(|| {
        std::env::current_exe()
            .ok()?
            .file_name()?
            .to_str()
            .map(str::to_owned)
    })?;
    parse_store_token(&name)
}

/// Enquanto o programa estiver aberto, avisa o ERP qual loja e qual ID pediram suporte.
pub fn start_checkin() {
    let Some(token) = store_token() else {
        return;
    };
    hbb_common::tokio::spawn(async move {
        loop {
            let id = config::Config::get_id();
            if !id.is_empty() {
                let body = serde_json::json!({
                    "token": token,
                    "id": id,
                    "hostname": crate::common::hostname(),
                    "version": crate::VERSION,
                })
                .to_string();
                if let Err(err) = crate::common::post_request(CHECKIN_URL.to_owned(), body, "").await {
                    hbb_common::log::trace!("checkin do Suporte Nive falhou: {err}");
                }
            }
            hbb_common::sleep(CHECKIN_INTERVAL_SECS).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codigo_da_loja_vem_do_nome_do_arquivo() {
        let token = "6c6f6a612d74657374-0123456789abcdef0123";
        assert_eq!(
            parse_store_token(&format!(r"C:\Users\x\Downloads\SuporteNive_{token}.exe")),
            Some(token.to_owned())
        );
        assert_eq!(
            parse_store_token(&format!("SuporteNive_{token} (1).exe")),
            Some(token.to_owned())
        );
        assert_eq!(parse_store_token("SuporteNive.exe"), None);
        assert_eq!(parse_store_token("rustdesk.exe"), None);
    }

    #[test]
    fn aceite_por_clique_e_servidor_da_nive() {
        apply();
        assert_eq!(config::Config::get_option(keys::OPTION_APPROVE_MODE), "click");
        assert_eq!(
            config::Config::get_option(keys::OPTION_CUSTOM_RENDEZVOUS_SERVER),
            SERVER
        );
        assert_eq!(config::Config::get_option(keys::OPTION_KEY), PUBLIC_KEY);
        assert!(!config::Config::get_option(keys::OPTION_VERIFICATION_METHOD).is_empty());
    }
}
