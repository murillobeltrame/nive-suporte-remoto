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

#[cfg(test)]
mod tests {
    use super::*;

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
