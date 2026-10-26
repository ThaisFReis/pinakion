#![no_std]
//! Processo: o caso em volta do sorteio, da denúncia ao veredito (T-048).
//!
//! O `kleroterion` decide **quem senta**. Este contrato conduz o resto, e não
//! decide nada que uma regra fixada antes do caso não decida:
//!
//! 1. `abrir` — com o rol já fechado para julgar alguém, o escrivão registra a
//!    denúncia. A pessoa julgada, o nível de gravidade e `k` vêm do
//!    `kleroterion`, não de quem chama.
//! 2. `comparecer` — cada jurado sentado confirma presença **pela conta de
//!    assento**, sem dizer quem é. O contrato só pergunta ao `kleroterion` se
//!    aquela conta sentou. Com o quórum, a sessão se instala; sem ele, no
//!    prazo, o caso é `adiar`.
//! 3. `registrar` — os autos: provas e atas do escrivão, a defesa da pessoa
//!    julgada (afastada, mas nunca calada), atas dos jurados presentes. Cada
//!    registro é o SHA-256 de um documento que fica fora da cadeia; quem tem
//!    o documento confere.
//! 4. `vincular_votacao` — a votação é do **Tessera**, o módulo de voto
//!    secreto anterior da mesma autora, usado sem modificação. O contrato só
//!    aceita uma votação do Tessera cuja lista de aptos é exatamente a dos
//!    jurados presentes: recalcula a raiz de Merkle com a regra do Tessera e
//!    compara com a que o Tessera guardou.
//! 5. `decidir` — lê o placar que o Tessera publicou e aplica a regra do nível:
//!    maioria do painel, ou dois terços a partir do nível fixado no construtor.
//!
//! O afastamento cautelar termina no veredito: `suspended` é o do
//! `kleroterion` enquanto o caso corre, e falso depois de decidido.

use soroban_sdk::{
    contract, contractclient, contracterror, contractevent, contractimpl, contracttype,
    panic_with_error, xdr::ToXdr, Address, Bytes, BytesN, Env, Vec, U256,
};

/// Separação de domínio da árvore de aptos do Tessera
/// (`tessera/core/src/merkle.rs` no commit 356c22f): folha, nó, vazio.
const DOM_FOLHA: u8 = 0x00;
const DOM_NO: u8 = 0x01;
const DOM_VAZIO: u8 = 0x02;
const VAZIO: &[u8] = b"TESSERA-V1-MERKLE-VAZIO";

// ---- o que se lê do kleroterion --------------------------------------------

/// Espelho de `kleroterion::Fechamento`: mesmos campos, mesmo XDR.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fechamento {
    pub ledger: u32,
    pub tempo: u64,
    pub rodada: u64,
    pub k: u32,
    pub revela_em: u32,
    pub raiz: U256,
    pub n: u32,
    pub gravidade: u32,
    pub acusado: Address,
    pub afastado_ate: u32,
}

/// Espelho de `kleroterion::Final`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Final {
    pub painel: Vec<U256>,
    pub completo: bool,
    pub ledger: u32,
}

#[contractclient(name = "KleroterionClient")]
pub trait Kleroterion {
    fn closing(env: Env) -> Option<Fechamento>;
    fn panel(env: Env) -> Option<Final>;
    fn seated(env: Env, assento: Address) -> bool;
    fn suspended(env: Env, endereco: Address) -> bool;
}

// ---- o que se lê do Tessera ------------------------------------------------

/// Espelho de `tessera::tipos::Pergunta` (commit 356c22f).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pergunta {
    pub opcoes: u32,
    pub confidencial: bool,
}

/// Espelho de `tessera::tipos::Proposta` (commit 356c22f).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposta {
    pub perguntas: Vec<Pergunta>,
    pub raiz_aptos: BytesN<32>,
    pub mesa: Vec<Address>,
    pub limiar: u32,
    pub abre_em: u32,
    pub fecha_em: u32,
    pub rodada: u64,
    pub rodada_abertura: u64,
    pub anel: bool,
    pub secoes: u32,
    pub limite_secao: u32,
}

#[contractclient(name = "TesseraClient")]
pub trait Tessera {
    fn proposta(env: Env, proposta: BytesN<32>) -> Option<Proposta>;
    fn resultado(env: Env, proposta: BytesN<32>) -> Option<Vec<u32>>;
}

// ---- tipos do processo -----------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Erro {
    /// O caso já foi aberto.
    JaAberto = 1,
    /// O rol do `kleroterion` ainda não fechou: não há quem julgar.
    RolAberto = 2,
    /// `k` abaixo do mínimo de votos que a votação secreta exige.
    PainelPequenoParaOSigilo = 3,
    /// O caso não está na fase que esta chamada pede.
    ForaDeFase = 4,
    /// O painel ainda não saiu.
    SemPainel = 5,
    /// Esta conta não sentou no painel.
    NaoSentou = 6,
    /// Esta conta já compareceu.
    JaCompareceu = 7,
    /// O prazo do quórum venceu.
    PrazoDoQuorumVencido = 8,
    /// O prazo do quórum ainda não venceu.
    PrazoDoQuorumAberto = 9,
    /// Quem chama não tem papel para este registro.
    SemPapel = 10,
    /// O Tessera não conhece esta votação.
    SemVotacao = 11,
    /// A cédula não é uma pergunta sigilosa de duas opções.
    CedulaErrada = 12,
    /// A lista de aptos da votação não é a dos jurados presentes.
    AptosNaoSaoOsPresentes = 13,
    /// Um membro da mesa da votação não é jurado presente.
    MesaDeFora = 14,
    /// A votação abriu antes de a sessão se instalar.
    VotacaoAnteriorASessao = 15,
    /// O Tessera ainda não publicou o resultado.
    SemResultado = 16,
}

/// Em que ponto o caso está.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Fase {
    /// Denúncia registrada; o painel se forma e os jurados comparecem.
    Aberto = 0,
    /// Quórum atingido: a sessão está instalada.
    Sessao = 1,
    /// A votação do Tessera está vinculada ao caso.
    Votacao = 2,
    /// O veredito saiu.
    Decidido = 3,
    /// O prazo do quórum venceu sem quórum.
    Adiado = 4,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Tipo {
    Denuncia = 0,
    Prova = 1,
    Defesa = 2,
    Ata = 3,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Papel {
    Escrivao = 0,
    Acusado = 1,
    Jurado = 2,
}

/// O caso: quem é julgado, em que nível, quantos assentos e o quórum.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Caso {
    pub acusado: Address,
    pub gravidade: u32,
    pub k: u32,
    pub quorum: u32,
    pub exigidos: u32,
    pub aberto_em: u32,
}

/// Um registro dos autos: o hash de um documento, quem o pôs e quando.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Registro {
    pub seq: u32,
    pub tipo: Tipo,
    pub papel: Papel,
    pub autor: Address,
    pub hash: BytesN<32>,
    pub ledger: u32,
}

/// O veredito: o placar do Tessera e a regra do nível.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Decisao {
    pub sim: u32,
    pub nao: u32,
    pub exigidos: u32,
    pub procedente: bool,
    pub ledger: u32,
}

#[contracttype]
enum Chave {
    Escrivao,
    Kleroterion,
    Tessera,
    PrazoQuorum,
    VotosMinimos,
    DoisTercosDesde,
    Caso,
    Fase,
    Presentes,
    Registros,
    SessaoEm,
    Votacao,
    Decisao,
}

// ---- eventos ---------------------------------------------------------------

/// A regra, publicada antes de qualquer caso.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Instituido {
    pub kleroterion: Address,
    pub tessera: Address,
    pub prazo_quorum: u32,
    pub votos_minimos: u32,
    pub dois_tercos_desde: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CasoAberto {
    #[topic]
    pub acusado: Address,
    pub gravidade: u32,
    pub k: u32,
    pub quorum: u32,
    pub exigidos: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Registrado {
    pub seq: u32,
    pub tipo: Tipo,
    pub papel: Papel,
    pub autor: Address,
    pub hash: BytesN<32>,
}

/// Uma conta de assento compareceu. Leva a conta, não o nome: o nome só
/// aparece quando o jurado revelar no `kleroterion`.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Compareceu {
    pub assento: Address,
    pub presentes: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessaoInstalada {
    pub presentes: u32,
    pub quorum: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Adiado {
    pub presentes: u32,
    pub quorum: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VotacaoVinculada {
    pub proposta: BytesN<32>,
    pub raiz_aptos: BytesN<32>,
    pub votantes: u32,
    pub fecha_em: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Decidido {
    pub sim: u32,
    pub nao: u32,
    pub exigidos: u32,
    pub procedente: bool,
    pub gravidade: u32,
}

#[contract]
pub struct Processo;

#[contractimpl]
impl Processo {
    /// `escrivao` abre o caso, registra provas e atas e vincula a votação;
    /// `kleroterion` é o rol deste caso; `tessera` é o contrato de voto;
    /// `prazo_quorum` é quantos ledgers, depois do painel, os jurados têm
    /// para comparecer; `votos_minimos` é o piso de sigilo da votação (o do
    /// Tessera é 5: abaixo disso ele não publica placar); a partir do nível
    /// `dois_tercos_desde` a decisão exige dois terços do painel.
    pub fn __constructor(
        env: Env,
        escrivao: Address,
        kleroterion: Address,
        tessera: Address,
        prazo_quorum: u32,
        votos_minimos: u32,
        dois_tercos_desde: u32,
    ) {
        let s = env.storage().instance();
        s.set(&Chave::Escrivao, &escrivao);
        s.set(&Chave::Kleroterion, &kleroterion);
        s.set(&Chave::Tessera, &tessera);
        s.set(&Chave::PrazoQuorum, &prazo_quorum);
        s.set(&Chave::VotosMinimos, &votos_minimos);
        s.set(&Chave::DoisTercosDesde, &dois_tercos_desde);
        s.set(&Chave::Presentes, &Vec::<Address>::new(&env));
        s.set(&Chave::Registros, &Vec::<Registro>::new(&env));
        Instituido {
            kleroterion,
            tessera,
            prazo_quorum,
            votos_minimos,
            dois_tercos_desde,
        }
        .publish(&env);
    }

    /// Abre o caso com a denúncia, depois de o rol fechar. A pessoa julgada,
    /// o nível e `k` são os do `kleroterion`. O quórum é a maioria do painel,
    /// e nunca menos que `votos_minimos`; a decisão exige a maioria, ou dois
    /// terços a partir de `dois_tercos_desde`.
    pub fn abrir(env: Env, denuncia: BytesN<32>) -> Caso {
        let escrivao = Self::escrivao(&env);
        escrivao.require_auth();
        if Self::fase(env.clone()).is_some() {
            panic_with_error!(&env, Erro::JaAberto);
        }
        let f = match Self::klero(&env).closing() {
            Some(f) => f,
            None => panic_with_error!(&env, Erro::RolAberto),
        };
        let i = env.storage().instance();
        let minimos: u32 = i.get(&Chave::VotosMinimos).unwrap();
        if f.k < minimos {
            panic_with_error!(&env, Erro::PainelPequenoParaOSigilo);
        }
        let desde: u32 = i.get(&Chave::DoisTercosDesde).unwrap();
        let caso = Caso {
            acusado: f.acusado.clone(),
            gravidade: f.gravidade,
            k: f.k,
            quorum: (f.k / 2 + 1).max(minimos),
            exigidos: if f.gravidade >= desde {
                (2 * f.k).div_ceil(3)
            } else {
                f.k / 2 + 1
            },
            aberto_em: env.ledger().sequence(),
        };
        i.set(&Chave::Caso, &caso);
        i.set(&Chave::Fase, &Fase::Aberto);
        CasoAberto {
            acusado: caso.acusado.clone(),
            gravidade: caso.gravidade,
            k: caso.k,
            quorum: caso.quorum,
            exigidos: caso.exigidos,
        }
        .publish(&env);
        Self::anotar(&env, Tipo::Denuncia, Papel::Escrivao, escrivao, denuncia);
        Self::manter_vivo(&env);
        caso
    }

    /// Um jurado confirma presença pela conta de assento. Só quem sentou,
    /// uma vez, e antes de o prazo do quórum vencer enquanto a sessão não se
    /// instalou. Com o quórum, a sessão se instala; depois dela, quem chega
    /// ainda conta, até a votação ser vinculada.
    pub fn comparecer(env: Env, assento: Address) -> u32 {
        assento.require_auth();
        let fase = Self::fase_ou_falha(&env);
        if fase != Fase::Aberto && fase != Fase::Sessao {
            panic_with_error!(&env, Erro::ForaDeFase);
        }
        let k = Self::klero(&env);
        let painel = match k.panel() {
            Some(p) => p,
            None => panic_with_error!(&env, Erro::SemPainel),
        };
        if fase == Fase::Aberto && env.ledger().sequence() > Self::prazo_do_quorum(&env, &painel) {
            panic_with_error!(&env, Erro::PrazoDoQuorumVencido);
        }
        if !k.seated(&assento) {
            panic_with_error!(&env, Erro::NaoSentou);
        }
        let i = env.storage().instance();
        let mut presentes = Self::presentes(env.clone());
        if presentes.contains(&assento) {
            panic_with_error!(&env, Erro::JaCompareceu);
        }
        presentes.push_back(assento.clone());
        i.set(&Chave::Presentes, &presentes);
        let n = presentes.len();
        Compareceu {
            assento,
            presentes: n,
        }
        .publish(&env);
        let caso = Self::caso(env.clone()).unwrap();
        if fase == Fase::Aberto && n >= caso.quorum {
            i.set(&Chave::Fase, &Fase::Sessao);
            i.set(&Chave::SessaoEm, &env.ledger().sequence());
            SessaoInstalada {
                presentes: n,
                quorum: caso.quorum,
            }
            .publish(&env);
        }
        Self::manter_vivo(&env);
        n
    }

    /// Vencido o prazo sem quórum, o caso é adiado. Qualquer um pode chamar.
    /// Um novo sorteio é um novo rol, e fica fora deste contrato.
    pub fn adiar(env: Env) {
        if Self::fase_ou_falha(&env) != Fase::Aberto {
            panic_with_error!(&env, Erro::ForaDeFase);
        }
        let painel = match Self::klero(&env).panel() {
            Some(p) => p,
            None => panic_with_error!(&env, Erro::SemPainel),
        };
        if env.ledger().sequence() <= Self::prazo_do_quorum(&env, &painel) {
            panic_with_error!(&env, Erro::PrazoDoQuorumAberto);
        }
        env.storage().instance().set(&Chave::Fase, &Fase::Adiado);
        Adiado {
            presentes: Self::presentes(env.clone()).len(),
            quorum: Self::caso(env.clone()).unwrap().quorum,
        }
        .publish(&env);
    }

    /// Põe um documento nos autos, pelo hash. Quem pode o quê:
    /// - o escrivão: provas, até a votação; atas, a qualquer momento;
    /// - a pessoa julgada: a defesa, até o veredito — afastada, nunca calada;
    /// - um jurado presente, pela conta de assento: atas, até o veredito.
    pub fn registrar(env: Env, autor: Address, tipo: Tipo, hash: BytesN<32>) -> u32 {
        autor.require_auth();
        let fase = Self::fase_ou_falha(&env);
        let antes_do_veredito = fase != Fase::Decidido && fase != Fase::Adiado;
        let antes_da_votacao = fase == Fase::Aberto || fase == Fase::Sessao;
        let caso = Self::caso(env.clone()).unwrap();
        let papel = if autor == Self::escrivao(&env) {
            Papel::Escrivao
        } else if autor == caso.acusado {
            Papel::Acusado
        } else if Self::presentes(env.clone()).contains(&autor) {
            Papel::Jurado
        } else {
            panic_with_error!(&env, Erro::SemPapel)
        };
        let pode = match (papel, tipo) {
            (Papel::Escrivao, Tipo::Prova) => antes_da_votacao,
            (Papel::Escrivao, Tipo::Ata) => true,
            (Papel::Acusado, Tipo::Defesa) => antes_do_veredito,
            (Papel::Jurado, Tipo::Ata) => antes_do_veredito,
            _ => false,
        };
        if !pode {
            panic_with_error!(&env, Erro::SemPapel);
        }
        let seq = Self::anotar(&env, tipo, papel, autor, hash);
        Self::manter_vivo(&env);
        seq
    }

    /// Vincula a votação do Tessera ao caso. Só aceita uma votação de uma
    /// pergunta sigilosa com duas opções (sim, não), aberta depois de a sessão
    /// se instalar, cuja lista de aptos é **exatamente** a dos jurados
    /// presentes, na ordem em que compareceram, e cuja mesa é de presentes.
    pub fn vincular_votacao(env: Env, proposta: BytesN<32>) {
        Self::escrivao(&env).require_auth();
        if Self::fase_ou_falha(&env) != Fase::Sessao {
            panic_with_error!(&env, Erro::ForaDeFase);
        }
        let p = match Self::tessera(&env).proposta(&proposta) {
            Some(p) => p,
            None => panic_with_error!(&env, Erro::SemVotacao),
        };
        let uma_sigilosa_de_duas = p.perguntas.len() == 1
            && p.perguntas.get(0).unwrap()
                == Pergunta {
                    opcoes: 2,
                    confidencial: true,
                };
        if !uma_sigilosa_de_duas {
            panic_with_error!(&env, Erro::CedulaErrada);
        }
        let presentes = Self::presentes(env.clone());
        if p.raiz_aptos != Self::raiz_dos_aptos(env.clone(), presentes.clone()) {
            panic_with_error!(&env, Erro::AptosNaoSaoOsPresentes);
        }
        if p.mesa.iter().any(|m| !presentes.contains(&m)) {
            panic_with_error!(&env, Erro::MesaDeFora);
        }
        let sessao: u32 = env.storage().instance().get(&Chave::SessaoEm).unwrap();
        if p.abre_em < sessao {
            panic_with_error!(&env, Erro::VotacaoAnteriorASessao);
        }
        let i = env.storage().instance();
        i.set(&Chave::Votacao, &proposta);
        i.set(&Chave::Fase, &Fase::Votacao);
        VotacaoVinculada {
            proposta,
            raiz_aptos: p.raiz_aptos,
            votantes: presentes.len(),
            fecha_em: p.fecha_em,
        }
        .publish(&env);
        Self::manter_vivo(&env);
    }

    /// Lê o placar do Tessera e aplica a regra do nível: procedente se `sim`
    /// alcança os votos exigidos, contados sobre o painel inteiro — ausência
    /// não é voto a favor. Qualquer um pode chamar.
    pub fn decidir(env: Env) -> Decisao {
        if Self::fase_ou_falha(&env) != Fase::Votacao {
            panic_with_error!(&env, Erro::ForaDeFase);
        }
        let proposta: BytesN<32> = env.storage().instance().get(&Chave::Votacao).unwrap();
        let placar = match Self::tessera(&env).resultado(&proposta) {
            Some(r) if r.len() == 2 => r,
            _ => panic_with_error!(&env, Erro::SemResultado),
        };
        let caso = Self::caso(env.clone()).unwrap();
        let sim = placar.get(0).unwrap();
        let d = Decisao {
            sim,
            nao: placar.get(1).unwrap(),
            exigidos: caso.exigidos,
            procedente: sim >= caso.exigidos,
            ledger: env.ledger().sequence(),
        };
        let i = env.storage().instance();
        i.set(&Chave::Decisao, &d);
        i.set(&Chave::Fase, &Fase::Decidido);
        Decidido {
            sim: d.sim,
            nao: d.nao,
            exigidos: d.exigidos,
            procedente: d.procedente,
            gravidade: caso.gravidade,
        }
        .publish(&env);
        Self::manter_vivo(&env);
        d
    }

    /// A pessoa está afastada? É o afastamento do `kleroterion` enquanto o
    /// caso corre; termina no veredito ou no adiamento, e o que vem depois
    /// — a sanção — é de quem chama o módulo.
    pub fn suspended(env: Env, endereco: Address) -> bool {
        match Self::fase(env.clone()) {
            Some(Fase::Decidido) | Some(Fase::Adiado) => false,
            _ => Self::klero(&env).suspended(&endereco),
        }
    }

    /// A raiz de Merkle que o Tessera calcula para esta lista de aptos, todos
    /// com peso 1: folha `SHA-256(0x00 ‖ xdr(endereço) ‖ peso_be)`, nó
    /// `SHA-256(0x01 ‖ esq ‖ dir)`, completada até a potência de 2 com a folha
    /// vazia `SHA-256(0x02 ‖ "TESSERA-V1-MERKLE-VAZIO")`.
    pub fn raiz_dos_aptos(env: Env, aptos: Vec<Address>) -> BytesN<32> {
        let mut nivel: Vec<BytesN<32>> = Vec::new(&env);
        for a in aptos.iter() {
            let mut b = Bytes::from_array(&env, &[DOM_FOLHA]);
            b.append(&a.to_xdr(&env));
            b.extend_from_array(&1u32.to_be_bytes());
            nivel.push_back(env.crypto().sha256(&b).to_bytes());
        }
        if nivel.is_empty() {
            return Self::vazio(&env);
        }
        let alvo = nivel.len().next_power_of_two();
        while nivel.len() < alvo {
            nivel.push_back(Self::vazio(&env));
        }
        while nivel.len() > 1 {
            let mut acima = Vec::new(&env);
            for i in (0..nivel.len()).step_by(2) {
                let mut b = Bytes::from_array(&env, &[DOM_NO]);
                b.extend_from_array(&nivel.get(i).unwrap().to_array());
                b.extend_from_array(&nivel.get(i + 1).unwrap().to_array());
                acima.push_back(env.crypto().sha256(&b).to_bytes());
            }
            nivel = acima;
        }
        nivel.get(0).unwrap()
    }

    pub fn caso(env: Env) -> Option<Caso> {
        env.storage().instance().get(&Chave::Caso)
    }

    pub fn fase(env: Env) -> Option<Fase> {
        env.storage().instance().get(&Chave::Fase)
    }

    /// As contas de assento presentes, na ordem em que compareceram.
    pub fn presentes(env: Env) -> Vec<Address> {
        env.storage().instance().get(&Chave::Presentes).unwrap()
    }

    pub fn registros(env: Env) -> Vec<Registro> {
        env.storage().instance().get(&Chave::Registros).unwrap()
    }

    pub fn votacao(env: Env) -> Option<BytesN<32>> {
        env.storage().instance().get(&Chave::Votacao)
    }

    pub fn decisao(env: Env) -> Option<Decisao> {
        env.storage().instance().get(&Chave::Decisao)
    }

    // ---- internos ------------------------------------------------------

    fn anotar(env: &Env, tipo: Tipo, papel: Papel, autor: Address, hash: BytesN<32>) -> u32 {
        let i = env.storage().instance();
        let mut rs = Self::registros(env.clone());
        let seq = rs.len();
        rs.push_back(Registro {
            seq,
            tipo,
            papel,
            autor: autor.clone(),
            hash: hash.clone(),
            ledger: env.ledger().sequence(),
        });
        i.set(&Chave::Registros, &rs);
        Registrado {
            seq,
            tipo,
            papel,
            autor,
            hash,
        }
        .publish(env);
        seq
    }

    /// Os autos vivem na instância: um caso tem dezenas de registros, não
    /// milhares, e a instância cabe 64 KiB. Cada escrita leva o TTL ao teto
    /// da rede, para os autos não arquivarem no meio do caso.
    fn manter_vivo(env: &Env) {
        let teto = env.storage().max_ttl();
        env.storage().instance().extend_ttl(teto, teto);
    }

    fn prazo_do_quorum(env: &Env, painel: &Final) -> u32 {
        let prazo: u32 = env.storage().instance().get(&Chave::PrazoQuorum).unwrap();
        painel.ledger.saturating_add(prazo)
    }

    fn fase_ou_falha(env: &Env) -> Fase {
        match Self::fase(env.clone()) {
            Some(f) => f,
            None => panic_with_error!(env, Erro::ForaDeFase),
        }
    }

    fn vazio(env: &Env) -> BytesN<32> {
        let mut b = Bytes::from_array(env, &[DOM_VAZIO]);
        b.extend_from_slice(VAZIO);
        env.crypto().sha256(&b).to_bytes()
    }

    fn escrivao(env: &Env) -> Address {
        env.storage().instance().get(&Chave::Escrivao).unwrap()
    }

    fn klero(env: &Env) -> KleroterionClient<'_> {
        let a: Address = env.storage().instance().get(&Chave::Kleroterion).unwrap();
        KleroterionClient::new(env, &a)
    }

    fn tessera(env: &Env) -> TesseraClient<'_> {
        let a: Address = env.storage().instance().get(&Chave::Tessera).unwrap();
        TesseraClient::new(env, &a)
    }
}

mod test;
