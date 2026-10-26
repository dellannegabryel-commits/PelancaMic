## Servidor (PC) — `Docs/pc/README.md`

**Módulo:** Servidor Desktop (Rust + Tauri v3)
**Objetivo:** Receber pacotes de áudio raw via rede local, gerenciar o Jitter Buffer adaptativo e injetar os dados no driver de áudio virtual do sistema operacional com a menor latência possível.

### Arquitetura de Rede (Backend Rust)

O servidor opera escutando ativamente em três portas distintas, utilizando `tokio` para assincronismo de rede:

* **Descoberta (UDP 41234 - Broadcast):**
* **Ação:** Envia um *beacon* a cada 1 segundo em `255.255.255.255:41234`.
* **Payload (JSON):** `{"app":"pelancamic","v":1,"name":"DESKTOP-NOME","audioPort":41235,"ctrlPort":41236}`


* **Controle (TCP/WS 41236 - WebSocket):**
* **Ação:** Servidor WebSocket aguardando conexão do cliente para troca de estados.
* **Comandos Aceitos:** `hello`, `mute` (boolean), `volume` (float 0.0 a 4.0), `bye`.


* **Áudio (UDP 41235 - Unicast):**
* **Ação:** Recebe o fluxo unidirecional de áudio do Android.
* **Processamento:** Deserializa o header binário de 16-bytes (Big-Endian). Se o `magic` (`0x4D53`) e a `version` (1) baterem, extrai o payload e o encaminha para o Jitter Buffer.



### Jitter Buffer e Sink (cpal)

1. **Buffer Adaptativo:** Mantém um alvo flutuante de 3 a 8 frames (30ms a 80ms) para absorver a variação (jitter) do Wi-Fi. Se a rede estiver perfeita, o alvo cai. Se pacotes chegarem fora de ordem, o `seq` do header organiza a fila.
2. **Audio Sink (`cpal` / `PortAudio`):** O Rust localiza o cabo virtual (ex: `CABLE Input (VB-Audio)`) usando a API do SO e escreve os blocos de 480 samples em tempo real a 48kHz.

---
