# Suporte Nive

Cópia (fork) do [RustDesk](https://github.com/rustdesk/rustdesk) usada pela Nive Sistemas
para atender lojistas a distância. Licença AGPL-3.0, igual ao original: este código precisa
continuar público.

## O que muda em relação ao RustDesk

Tudo fica em `src/nive.rs`, chamado por uma linha em `common::load_custom_client`:

- **Servidor fixo**: `suporte.nivesistemas.com.br` (hbbs/hbbr na VPS) e a chave pública dele.
  O usuário não consegue trocar pela tela de configurações.
- **Nome**: "Suporte Nive".
- **Aceite só por clique**: não existe senha. Toda conexão abre um aviso no computador do
  cliente e só segue se ele clicar em **Aceitar**.
- **Sem atualização do RustDesk original**, que trocaria servidor e nome.
- **Ícones** da Nive em `res/` e `flutter/windows/runner/resources/app_icon.ico`.

## Branches

- `nive` (padrão): o Suporte Nive.
- `master`: espelho do RustDesk original, para receber atualizações.

Para trazer uma versão nova do RustDesk:

```bash
git fetch upstream
git checkout master && git merge --ff-only upstream/master && git push origin master
git checkout nive && git merge master
```

Conflitos costumam aparecer só nos ícones (manter os da Nive).

## Gerar o executável

Workflow **Suporte Nive · Windows** (`.github/workflows/nive-windows.yml`), só Windows x64:

- Manual: Actions → Suporte Nive · Windows → Run workflow. O `SuporteNive.exe` sai como
  artefato da execução.
- Versão publicada: `git tag nive-1.0.0 && git push origin nive-1.0.0`. Cria uma Release com
  o `SuporteNive.exe`.

O build leva perto de uma hora na primeira vez (depois o cache ajuda).

O executável não é assinado: o Windows mostra o aviso do SmartScreen ("Executar assim mesmo")
até a Nive ter um certificado de assinatura de código.

## Como o lojista usa

1. Baixa o `SuporteNive.exe` pelo ERP (Ajuda → Suporte remoto).
2. Dá dois cliques. Não precisa instalar nem ser administrador.
3. Passa o ID que aparece na tela para o atendimento.
4. Clica em **Aceitar** quando a Nive conectar.
