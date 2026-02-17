use std::io::{self, Write};
use tokio::time::sleep;
use std::time::Duration;
use qrcode::{QrCode, render::unicode};

use torchat_paste_core::{AppState, Contact, TorManager, crypto::Fingerprint};

// Estrutura para manter a sessão na memória (Senha dos contatos)
struct UserSession {
    password: String, // Mantido na RAM apenas enquanto o app roda
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("TorChat-Paste - Mensageiro Anônimo & Seguro");
    println!("============================================");

    let state = AppState::new();

    // 1. Login e Criptografia
    let session = gerenciar_identidade(&state).await?;

    // 2. Inicialização do Tor
    inicializar_tor(&state).await?;

    // 3. Setup do Serviço Oculto (exemplo simplificado)
    criar_servico_oculto(&state).await?;

    loop {
        println!("\n--- Menu Principal ---");
        println!("1. Listar contatos (Criptografado)");
        println!("2. Adicionar contato (Seguro)");
        println!("3. Conversar");
        println!("4. Compartilhar endereço (OnionShare + QR)");
        println!("5. Sair");
        print!("Escolha: ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        
        match choice.trim() {
            "1" => listar_contatos(&state, &session).await?,
            "2" => adicionar_contato(&state, &session).await?,
            "3" => conversar(&state, &session).await?,
            "4" => compartilhar_endereco(&state).await?,
            "5" => {
                println!("Limpando memória e encerrando...");
                break;
            }
            _ => println!("Opção inválida."),
        }
    }

    Ok(())
}

async fn gerenciar_identidade(state: &AppState) -> Result<UserSession, Box<dyn std::error::Error>> {
    let storage = state.storage.as_ref();
    let password;

    if storage.has_identity() {
        println!("\nIdentidade criptografada detectada.");
        print!("Digite sua senha para liberar o cofre: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        password = input.trim().to_string();

        match storage.load_identity(&password) {
            Ok(_) => println!("Cofre aberto com sucesso!"),
            Err(e) => {
                eprintln!("Acesso negado: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("\nBem-vindo! Vamos criar seu cofre seguro.");
        print!("Crie uma senha forte (ela criptografará tudo): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        password = input.trim().to_string();

        match storage.create_identity(&password) {
            Ok(fp) => {
                println!("Identidade criada! Fingerprint: {}", fp.formatted());
            },
            Err(e) => {
                eprintln!("Erro fatal: {}", e);
                std::process::exit(1);
            }
        }
    }
    
    // Retorna a sessão com a senha para uso nas operações de contato
    Ok(UserSession { password })
}

async fn inicializar_tor(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nConectando à rede Tor...");
    let mut tor_manager = state.tor_manager.write().await;
    match tor_manager.bootstrap().await {
        Ok(_) => {
            println!("Conectado à rede Onion.");
            Ok(())
        },
        Err(e) => Err(e.into()),
    }
}

async fn criar_servico_oculto(_state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    // Implementação real iria aqui.
    Ok(())
}

async fn listar_contatos(state: &AppState, session: &UserSession) -> Result<(), Box<dyn std::error::Error>> {
    // Carrega contatos usando a senha da sessão
    let contacts = state.storage.load_contacts(&session.password)?;
    
    if contacts.is_empty() {
        println!("  (Nenhum contato no cofre)");
    } else {
        println!("--- Contatos no Cofre ---");
        for c in contacts {
            println!("  [{}] {}", c.nickname, c.address);
        }
    }
    Ok(())
}

async fn adicionar_contato(state: &AppState, session: &UserSession) -> Result<(), Box<dyn std::error::Error>> {
    print!("\nEndereço .onion do contato: ");
    io::stdout().flush()?;
    let mut address = String::new();
    io::stdin().read_line(&mut address)?;
    let address = address.trim().to_string();

    if let Err(e) = TorManager::validate_onion_address(&address) {
        eprintln!("Erro: {}", e);
        return Ok(());
    }

    print!("Apelido: ");
    io::stdout().flush()?;
    let mut nickname = String::new();
    io::stdin().read_line(&mut nickname)?;
    let nickname = nickname.trim().to_string();

    print!("Fingerprint (opcional): ");
    io::stdout().flush()?;
    let mut fp_input = String::new();
    io::stdin().read_line(&mut fp_input)?;
    
    let fingerprint = if !fp_input.trim().is_empty() {
        Fingerprint::new(fp_input.trim().replace('-', ""))
    } else {
        // Gera um dummy se não tiver (em prod não deveria permitir)
        Fingerprint::new("0000".to_string()) 
    };

    // Salva usando a senha da sessão
    state.storage.add_contact(&address, &nickname, fingerprint, &session.password)?;
    println!("Contato criptografado e salvo!");
    Ok(())
}

async fn conversar(state: &AppState, _session: &UserSession) -> Result<(), Box<dyn std::error::Error>> {
    // Lógica de chat simplificada
    print!("Endereço para conectar: ");
    io::stdout().flush()?;
    let mut address = String::new();
    io::stdin().read_line(&mut address)?;
    let address = address.trim();

    println!("Estabelecendo circuito Tor para {}...", address);
    let tor_manager = state.tor_manager.read().await;
    match tor_manager.connect_to_onion(address, 80).await {
        Ok(_) => println!("Conexão TCP estabelecida! (Chat P2P iniciaria aqui)"),
        Err(e) => println!("Falha ao conectar: {}", e),
    }
    Ok(())
}

fn mostrar_qr_code(conteudo: &str) {
    let code = QrCode::new(conteudo).unwrap();
    let image = code.render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    
    println!("\n--- ESCANEIE PARA ADICIONAR ---");
    println!("{}", image);
    println!("-------------------------------\n");
}

async fn compartilhar_endereco(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let meu_id = "v2c_exemplo_id_usuario".to_string(); // Pegar do storage real
    let mut tor_manager = state.tor_manager.write().await;

    println!("\nGerando link OnionShare temporário...");
    match tor_manager.create_ephemeral_sharing_service(meu_id).await {
        Ok(onion) => {
            let link = format!("http://{}", onion);
            println!("Link Ativo: {}", link);
            mostrar_qr_code(&link);
            
            println!("Pressione ENTER para destruir o link.");
            tokio::task::spawn_blocking(move || {
                let mut s = String::new();
                std::io::stdin().read_line(&mut s).ok();
            }).await?;
            
            tor_manager.shutdown_hidden_service();
            println!("Link destruído.");
        }
        Err(e) => eprintln!("Erro: {}", e),
    }
    Ok(())
}
