use std::io::{self, Write};
use std::time::Duration;
use tokio::time::sleep;
use rand::Rng;
use futures::stream::StreamExt;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, StatusCode};
use hyper::server::conn::Http;
use torchat_paste_core::{AppState, Contact, TorManager, crypto::Fingerprint};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("TorChat-Paste - Mensageiro Anônimo");
    println!("==================================");

    let state = AppState::new();

    gerenciar_identidade(&state).await?;
    inicializar_tor(&state).await?;
    criar_servico_oculto(&state).await?;

    loop {
        println!("\n--- Menu Principal ---");
        println!("1. Listar contatos");
        println!("2. Adicionar contato");
        println!("3. Conversar com contato");
        println!("4. Compartilhar endereço (OnionShare)");
        println!("5. Sair");
        print!("Escolha: ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        match choice.trim() {
            "1" => listar_contatos(&state).await?,
            "2" => adicionar_contato(&state).await?,
            "3" => conversar(&state).await?,
            "4" => compartilhar_endereco(&state).await?,
            "5" => {
                println!("Encerrando...");
                break;
            }
            _ => println!("Opção inválida."),
        }
    }

    Ok(())
}

async fn gerenciar_identidade(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let storage = state.storage.as_ref();

    if storage.has_identity() {
        println!("Identidade existente encontrada. Digite sua senha:");
        print!("Senha: ");
        io::stdout().flush()?;
        let mut password = String::new();
        io::stdin().read_line(&mut password)?;
        let password = password.trim();

        match storage.load_identity(password) {
            Ok(keypair) => {
                println!("Identidade carregada com sucesso!");
                println!("Chave pública: {}", keypair.public_key);
            }
            Err(e) => {
                eprintln!("Falha ao carregar identidade: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("Nenhuma identidade encontrada. Vamos criar uma nova.");
        print!("Defina uma senha para proteger sua identidade: ");
        io::stdout().flush()?;
        let mut password = String::new();
        io::stdin().read_line(&mut password)?;
        let password = password.trim();

        match storage.create_identity(password) {
            Ok(fingerprint) => {
                println!("Identidade criada com sucesso!");
                println!("Fingerprint da sua chave pública: {}", fingerprint.formatted());
                println!("Guarde este fingerprint e compartilhe com seus contatos para verificação.");
            }
            Err(e) => {
                eprintln!("Falha ao criar identidade: {}", e);
                std::process::exit(1);
            }
        }
    }
    Ok(())
}

async fn inicializar_tor(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nInicializando Tor...");
    let mut tor_manager = state.tor_manager.write().await;
    match tor_manager.bootstrap().await {
        Ok(_) => {
            println!("Tor pronto para uso.");
            Ok(())
        }
        Err(e) => {
            eprintln!("Erro ao inicializar Tor: {}", e);
            std::process::exit(1);
        }
    }
}

async fn criar_servico_oculto(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let mut tor_manager = state.tor_manager.write().await;
    if tor_manager.get_onion_address().is_none() {
        println!("Criando serviço oculto...");
        match tor_manager.create_hidden_service().await {
            Ok(onion) => {
                println!("Seu endereço onion: {}", onion);
            }
            Err(e) => {
                eprintln!("Falha ao criar serviço oculto: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!(
            "Serviço oculto já ativo: {}",
            tor_manager.get_onion_address().unwrap()
        );
    }
    Ok(())
}

async fn listar_contatos(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let contacts = state.contacts.read().await;
    if contacts.is_empty() {
        println!("Nenhum contato salvo.");
    } else {
        println!("Contatos:");
        for (addr, contact) in contacts.iter() {
            println!(
                "  {} - {} (online: {})",
                addr, contact.nickname, contact.online
            );
        }
    }
    Ok(())
}

async fn adicionar_contato(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    println!("Adicionar novo contato:");

    print!("Endereço onion: ");
    io::stdout().flush()?;
    let mut address = String::new();
    io::stdin().read_line(&mut address)?;
    let address = address.trim();

    if let Err(e) = TorManager::validate_onion_address(address) {
        eprintln!("Endereço inválido: {}", e);
        return Ok(());
    }

    print!("Apelido: ");
    io::stdout().flush()?;
    let mut nickname = String::new();
    io::stdin().read_line(&mut nickname)?;
    let nickname = nickname.trim();

    print!("Fingerprint (deixe em branco para pular verificação): ");
    io::stdout().flush()?;
    let mut fp_input = String::new();
    io::stdin().read_line(&mut fp_input)?;
    let fp_input = fp_input.trim();

    // Converte o fingerprint se fornecido
    let fingerprint = if !fp_input.is_empty() {
        // Remove hífens se houver e cria um Fingerprint
        let clean = fp_input.replace('-', "");
        Some(Fingerprint::new(clean))
    } else {
        None
    };

    // Adiciona ao storage persistente apenas se tiver fingerprint
    if let Some(fp) = &fingerprint {
        state.storage.add_contact(address, nickname, fp.clone())?;
    } else {
        println!("Aviso: sem fingerprint, a conexão não será verificada. Contato não será salvo permanentemente.");
    }

    // Adiciona ao hashmap em memória (sempre)
    {
        let mut contacts = state.contacts.write().await;
        contacts.insert(address.to_string(), Contact::new(address.to_string(), nickname.to_string()));
    }

    println!("Contato adicionado com sucesso.");
    Ok(())
}

async fn conversar(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    println!("Iniciar conversa:");
    listar_contatos(state).await?;

    print!("Digite o endereço do contato: ");
    io::stdout().flush()?;
    let mut address = String::new();
    io::stdin().read_line(&mut address)?;
    let address = address.trim();

    let contact = {
        let contacts = state.contacts.read().await;
        contacts.get(address).cloned()
    };

    if let Some(contact) = contact {
        println!("Conectando a {}...", contact.address);

        let tor_manager = state.tor_manager.read().await;
        match tor_manager.connect_to_onion(&contact.address, 8080).await {
            Ok(stream) => {
                println!("Conexão TCP estabelecida!");
                // Aqui viria a lógica de handshake e troca de mensagens
                drop(stream);
                println!("(Handshake ainda não implementado)");
            }
            Err(e) => {
                eprintln!("Falha na conexão: {}", e);
            }
        }
    } else {
        println!("Contato não encontrado.");
    }

    Ok(())
}

/// Compartilha o endereço onion principal criando um serviço efêmero (estilo OnionShare)
async fn compartilhar_endereco(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let meu_onion = {
        let tor_manager = state.tor_manager.read().await;
        tor_manager.get_onion_address().cloned()
    };

    match meu_onion {
        Some(onion) => {
            println!("Seu endereço onion principal: {}", onion);
            println!("Criando link de compartilhamento temporário via OnionShare...");

            // Gera uma porta local aleatória
            let mut rng = rand::thread_rng();
            let local_port = rng.gen_range(10000..20000);

            // Cria um serviço onion efêmero apontando para essa porta
            let (onion_ephemeral, mut requests) = {
                let tor_manager = state.tor_manager.write().await;
                tor_manager.create_ephemeral_sharing_service(&onion).await?
            };

            let link = format!("http://{}/", onion_ephemeral);
            println!("Link temporário (válido por 5 minutos): {}", link);
            println!("Compartilhe este link com seu contato por um canal seguro.");

            // Função auxiliar para servir HTTP em uma stream
            async fn serve_http(
                stream: impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
                meu_onion: String,
            ) -> Result<(), hyper::Error> {
                let service = service_fn(move |_req: Request<Body>| {
                    let meu_onion = meu_onion.clone();
                    async move {
                        Ok::<_, hyper::Error>(Response::new(Body::from(meu_onion)))
                    }
                });
                Http::new().serve_connection(stream, service).await
            }

            // Processa as requisições do serviço onion em background
            let handle = tokio::spawn(async move {
                while let Some(request) = requests.next().await {
                    match request.accept().await {
                        Ok(stream) => {
                            let onion_clone = onion.clone();
                            tokio::spawn(async move {
                                if let Err(e) = serve_http(stream, onion_clone).await {
                                    eprintln!("Erro ao servir HTTP: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("Erro ao aceitar requisição onion: {}", e);
                        }
                    }
                }
            });

            println!("Aguardando compartilhamento por até 5 minutos...");
            println!("Pressione Enter para cancelar manualmente.");

            // Aguarda 5 minutos ou interrupção do usuário
            tokio::select! {
                _ = sleep(Duration::from_secs(300)) => {
                    println!("Tempo esgotado. Link expirado.");
                }
                _ = async {
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).await.ok();
                } => {
                    println!("Compartilhamento cancelado.");
                }
            }

            // Encerra a task de processamento
            handle.abort();
            // O serviço onion efêmero será dropado ao sair do escopo
            println!("Link de compartilhamento desativado.");
            Ok(())
        }
        None => {
            eprintln!("Serviço oculto principal não está ativo.");
            Ok(())
        }
    }
}
