"""Recomputação do sorteio do Soroban, do zero, sem crate nenhuma.

Receita lida do host (rs-soroban-env):
  1. chave = HMAC-SHA256(SALT, semente)            [crypto/mod.rs: unbias_prng_seed]
  2. ChaCha20Rng::from_seed(chave)                 [rand_chacha 0.3.1]
  3. Fisher-Yates de rand 0.8.5 (SliceRandom::shuffle), com gen_index/Lemire
SALT = sha256("Public Global Stellar Network ; September 2015")
"""
import hmac, hashlib, struct, sys

SALT = bytes.fromhex("7ac33997544e3175d266bd022439b22cdb16508c01163f26e5cb2a3e1045a979")
M32 = 0xFFFFFFFF

def rotl(v, n): return ((v << n) | (v >> (32 - n))) & M32

def qr(s, a, b, c, d):
    s[a] = (s[a] + s[b]) & M32; s[d] = rotl(s[d] ^ s[a], 16)
    s[c] = (s[c] + s[d]) & M32; s[b] = rotl(s[b] ^ s[c], 12)
    s[a] = (s[a] + s[b]) & M32; s[d] = rotl(s[d] ^ s[a], 8)
    s[c] = (s[c] + s[d]) & M32; s[b] = rotl(s[b] ^ s[c], 7)

def chacha20_block(key32, counter, nonce12):
    const = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574]
    k = list(struct.unpack("<8I", key32))
    n = list(struct.unpack("<3I", nonce12))
    st = const + k + [counter & M32] + n
    w = st[:]
    for _ in range(10):
        qr(w, 0, 4, 8, 12); qr(w, 1, 5, 9, 13); qr(w, 2, 6, 10, 14); qr(w, 3, 7, 11, 15)
        qr(w, 0, 5, 10, 15); qr(w, 1, 6, 11, 12); qr(w, 2, 7, 8, 13); qr(w, 3, 4, 9, 14)
    return [(w[i] + st[i]) & M32 for i in range(16)]

class ChaCha20Rng:
    """Emite u32 na ordem em que o rand_chacha os consome."""
    def __init__(self, key32):
        self.key, self.counter, self.buf, self.pos = key32, 0, [], 0
    def next_u32(self):
        if self.pos >= len(self.buf):
            self.buf = chacha20_block(self.key, self.counter, b"\x00" * 12)
            self.counter += 1
            self.pos = 0
        v = self.buf[self.pos]; self.pos += 1
        return v

def gen_index(rng, ubound):
    """rand 0.8.5: UniformInt<u32>::sample_single, método de Lemire com rejeição."""
    rangе = ubound  # low=0, high=ubound exclusivo
    if rangе == 0:
        return rng.next_u32()
    zone = ((rangе << (32 - rangе.bit_length())) - 1) & M32 if rangе else M32
    # (range << leading_zeros) - 1, com leading_zeros de 32 bits
    lz = 32 - rangе.bit_length()
    zone = ((rangе << lz) & M32)
    zone = (zone - 1) & M32
    while True:
        v = rng.next_u32()
        prod = v * rangе
        hi, lo = prod >> 32, prod & M32
        if lo <= zone:
            return hi

def shuffle(lst, rng):
    """rand 0.8.5 SliceRandom::shuffle — Fisher-Yates de trás para a frente."""
    for i in range(len(lst) - 1, 0, -1):
        j = gen_index(rng, i + 1)
        lst[i], lst[j] = lst[j], lst[i]

def sortear(seed_hex, n, k):
    key = hmac.new(SALT, bytes.fromhex(seed_hex), hashlib.sha256).digest()
    rng = ChaCha20Rng(key)
    roll = list(range(n))
    shuffle(roll, rng)
    return roll[:k]

def painel(seed_hex, rol, inelegiveis, k):
    """O painel do kleroterion: embaralha a ordem do rol inteiro e senta os `k`
    primeiros que não estão em `inelegiveis` (impedidos ou sem bilhete).
    Não há novo sorteio para substituir: o substituto é o próximo da ordem."""
    fora = set(inelegiveis)
    ordem = sortear(seed_hex, len(rol), len(rol))
    sentados = [rol[i] for i in ordem if rol[i] not in fora]
    if len(sentados) < k:
        raise ValueError(f"so {len(sentados)} elegiveis para {k} assentos")
    return sentados[:k]

if __name__ == "__main__":
    print(sortear(sys.argv[1], int(sys.argv[2]), int(sys.argv[3])))
