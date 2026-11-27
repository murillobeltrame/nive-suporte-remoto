// Personalização do Suporte Nive: servidor próprio, nome e aceite só por clique.
// Fica em módulo separado para que as atualizações vindas do RustDesk original
// entrem sem conflito; o único gancho é a chamada em `common::load_custom_client`.

use base::config::keys;
use hbb_common::config;
use std::sync::atomic::{AtomicBool, Ordering};

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

    // Os cartões de ajuda ficam à mostra: no Windows é ali que aparece o botão "Instalar",
    // que deixa o Suporte Nive ao lado do relógio (texto em `translate`).
    let mut builtin = config::BUILTIN_SETTINGS.write().unwrap();
    for k in [
        keys::OPTION_HIDE_SERVER_SETTINGS,
        keys::OPTION_HIDE_PROXY_SETTINGS,
    ] {
        builtin.insert(k.to_owned(), "Y".to_owned());
    }
    drop(builtin);

    // Sem porta de entrada no Windows: abrir uma (conexão direta ou rede local) dispara o
    // aviso do firewall, que pede administrador. A conexão vai sempre pelo relay da Nive.
    config::HARD_SETTINGS
        .write()
        .unwrap()
        .insert("disable-tcp-listen".to_owned(), "Y".to_owned());
}

/// Textos próprios do Suporte Nive, por cima da tradução do RustDesk.
pub fn translate(name: &str) -> Option<String> {
    match name {
        "install_tip" => Some(
            "Quer o Suporte Nive sempre à mão? Instale neste computador: ele fica ao lado do \
             relógio e abre junto com o Windows. A Nive só conecta depois que você clicar em Aceitar."
                .to_owned(),
        ),
        _ => None,
    }
}

const CHECKIN_URL: &str = "https://erp.nivesistemas.com.br/api/public/suporte-remoto/checkin";
const CHECKIN_INTERVAL_SECS: u64 = 60;
/// Código da loja guardado na configuração: depois de instalado, o arquivo deixa de ter o
/// código no nome. A instalação copia a configuração do usuário para o serviço.
const STORE_TOKEN_OPTION: &str = "nive-store-token";

/// Este processo tem a janela do Suporte Nive aberta.
static WINDOW_OPEN: AtomicBool = AtomicBool::new(false);
static BACKGROUND_STARTED: AtomicBool = AtomicBool::new(false);

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

fn exe_store_token() -> Option<String> {
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

fn is_installed() -> bool {
    #[cfg(windows)]
    {
        crate::platform::is_installed()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// `open`: a janela está aberta, ou seja, o lojista está pedindo suporte agora. Sem janela
/// (instalado, só o ícone ao lado do relógio) o computador apenas aparece como disponível.
fn checkin_body(token: &str, id: &str, open: bool) -> String {
    serde_json::json!({
        "token": token,
        "id": id,
        "hostname": crate::common::hostname(),
        "version": crate::VERSION,
        "open": open,
        "installed": is_installed(),
    })
    .to_string()
}

/// Janela aberta: avisa o ERP a cada minuto qual loja e qual ID pediram suporte.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn start_window_checkin() {
    if WINDOW_OPEN.swap(true, Ordering::SeqCst) {
        return;
    }
    let from_exe = exe_store_token();
    if let Some(token) = &from_exe {
        // Gravado antes de qualquer instalação, que leva esta configuração para o serviço.
        config::Config::set_option(STORE_TOKEN_OPTION.to_owned(), token.clone());
    }
    std::thread::spawn(move || {
        // Tempo para o servidor local subir e responder o ID.
        std::thread::sleep(std::time::Duration::from_secs(3));
        if is_installed() {
            // Já instalado: quem guarda a configuração é o serviço, não esta janela.
            if let Some(token) = &from_exe {
                if crate::ipc::get_options().get(STORE_TOKEN_OPTION) != Some(token) {
                    crate::ipc::set_option(STORE_TOKEN_OPTION, token);
                }
            }
        }
        loop {
            let token = from_exe
                .clone()
                .or_else(|| crate::ipc::get_options().get(STORE_TOKEN_OPTION).cloned())
                .unwrap_or_default();
            let id = crate::ipc::get_id();
            if !token.is_empty() && !id.is_empty() {
                let body = checkin_body(&token, &id, true);
                if let Err(err) = crate::common::post_request_sync(CHECKIN_URL.to_owned(), body, "") {
                    hbb_common::log::trace!("checkin do Suporte Nive falhou: {err}");
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(CHECKIN_INTERVAL_SECS));
        }
    });
}

/// Serviço instalado, sem janela: mantém o computador visível para a Nive, sem pedir suporte.
/// No executável avulso o servidor roda dentro da própria janela, que já avisa sozinha.
pub fn start_checkin() {
    if WINDOW_OPEN.load(Ordering::SeqCst) || BACKGROUND_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    hbb_common::tokio::spawn(async move {
        loop {
            let token = config::Config::get_option(STORE_TOKEN_OPTION);
            let id = config::Config::get_id();
            if !token.is_empty() && !id.is_empty() {
                let body = checkin_body(&token, &id, false);
                if let Err(err) = crate::common::post_request(CHECKIN_URL.to_owned(), body, "").await {
                    hbb_common::log::trace!("checkin do Suporte Nive falhou: {err}");
                }
            }
            hbb_common::sleep(CHECKIN_INTERVAL_SECS as f32).await;
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

    #[test]
    fn aviso_diz_se_a_janela_esta_aberta() {
        let body: serde_json::Value =
            serde_json::from_str(&checkin_body("codigo", "123456789", false)).unwrap();
        assert_eq!(body["open"], false);
        assert_eq!(body["id"], "123456789");
        assert!(translate("install_tip").unwrap().contains("relógio"));
        assert_eq!(translate("Install"), None);
    }
}
